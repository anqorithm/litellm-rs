# Tech Spec

## Linked Issue

GH-1113 / #1113

## Product Spec

Link to `product.md`.

## Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| Provider trait/facade | `src/core/traits/provider/llm_provider/trait_definition.rs:453-475`, `src/core/providers/mod.rs:562-579` | Both public boundaries already return `Result<f64, ProviderError>`. | Typed outer contract exists; provider implementations must stop manufacturing successful zero costs. |
| Vertex provider calculator | `src/core/providers/vertex_ai/client.rs:285-335,408-428` | `models()` carries a separate static price table; `calculate_cost` uses substring branches and returns `Ok(0.0)` for unknown models. | Duplicate authority and silent zero root cause. GH1112 is expected to remove the static model table before this implementation. |
| Vertex Gemini helper | `src/core/providers/vertex_ai/gemini/mod.rs:341-361` | Public helper owns per-million hard-coded rates and defaults unknown models to Flash. | Second duplicate price table and wrong-model success. |
| Gemini public helpers | `src/core/providers/gemini/provider.rs:115-123,388-398`, `src/core/providers/gemini/models/mod.rs:288-373` | Inherent/basic/multimodal helpers return `Option<f64>`; trait implementation converts `None` to `Ok(0.0)`; registry fallback has its own per-1k math. | Public failure semantics and calculation authority diverge. |
| Canonical pricing authority | `src/core/pricing_service/authority.rs:24-190,193-247,303-399,421-490`, `src/core/pricing_service/service.rs:108-183` | Loaded provider-aware APIs return `Result`, but shared resolver still permits alias/fuzzy matching and converts provider-local per-1k values. | GH1113 must add an exact Google path without changing unrelated provider compatibility. |
| Runtime reservation/settlement | `src/server/routes/ai/spend/pricing.rs`, `src/server/routes/ai/spend/completion.rs`, `src/server/routes/ai/chat_streaming.rs`, `src/server/routes/ai/spend.rs` | Live paths use runtime `PricingService`, convert misses to model-not-priced errors, and share budget/spend identities. | Required parity and pre-upstream fail-closed evidence. |
| Explicit unpriced policy | `src/config/models/gateway.rs:242-288`, `src/server/routes/ai/spend/unpriced.rs:9-211`, `src/server/middleware/metrics.rs` | Default is `Reject`; `AllowUnpriced` computes fallback cost, records metrics/logs, and writes `UsageRecord::unpriced` when a key exists. | Preserve as explicit audited policy only; never move it into provider helpers. |
| GH1112 dependency | `specs/GH1112/product.md`, `specs/GH1112/tech.md`, `specs/GH1112/tasks.md` and its eventual merged `src/core/providers/google/models/**` | Defines exact canonical ID, per-surface availability, and a crate-private neutral catalog; deliberately leaves pricing behavior to GH1113. | Implementation must re-anchor to the merged API rather than guess or recreate a registry. |

## Planned Change Manifest

```json
{
  "issue": 1113,
  "complete": true,
  "dependency_gate": "PR #1117 merged and exact GH1112 catalog API re-anchored",
  "paths": [
    "src/core/pricing_service/authority.rs",
    "src/core/pricing_service/authority_tests.rs",
    "src/core/providers/unified_provider_error.rs",
    "src/core/providers/unified_provider_methods.rs",
    "src/core/providers/gemini/models/mod.rs",
    "src/core/providers/gemini/provider.rs",
    "src/core/providers/gemini/provider_tests.rs",
    "src/core/providers/vertex_ai/client.rs",
    "src/core/providers/vertex_ai/client_tests.rs",
    "src/core/providers/vertex_ai/gemini/mod.rs",
    "src/core/providers/vertex_ai/tests.rs",
    "src/server/routes/ai/spend/unpriced.rs",
    "src/server/routes/ai/spend_runtime_pricing_tests.rs",
    "src/server/routes/ai/spend_tests.rs",
    "src/server/routes/ai/execution_retry_delay_tests.rs",
    "CHANGELOG.md"
  ],
  "spec_refs": [
    "B-001", "B-002", "B-003", "B-004", "B-005",
    "B-006", "B-007", "B-008", "B-009", "B-010"
  ]
}
```

`src/core/providers/google/models/registry.rs` 仅列作 dependency anchor，不是 writable manifest
path；本 issue 对 merged GH1112 catalog 只读。若 #1117 合并结果改名、拆分或缺少本 spec
要求的 exact read API，先更新 manifest、明确所需 catalog API change 并重跑 spec review，
不得由 pricing task 静默修改 canonical ID/availability owner。`spend/unpriced.rs` 的
message-prefix classifier 已被 fresh review 证明不满足 B-007，因此本 issue 必须修改它；
不得借此扩大 policy 或 persistence scope。

## 设计方案

### 1. 依赖重锚与完整 inventory

实现前 fetch 最新 `origin/main`，确认 #1117 已合并，并从 merged Google registry 读取：

- exact `GoogleModelId`/canonical ID；
- Developer 与 Vertex 独立 availability；
- 明确声明的 compatibility alias（若存在）。

不得复制 registry、用当前旧 `GeminiModelRegistry::from_model_name` 或从 pricing keys 反推
model validity。对所有 public Google token-cost surface 建 tracked inventory：

- `LLMProvider::calculate_cost` 与 `Provider::calculate_cost`；
- `GeminiProvider::calculate_cost`；
- `gemini::models::CostCalculator::{calculate_cost, calculate_multimodal_cost}`；
- `vertex_ai::gemini::GeminiCostCalculator::calculate_cost`；
- `VertexAIProvider::calculate_cost`。

Vertex image-generation/TTS cost helper 不在此 inventory。source guard 必须在新增 Google
public token-cost helper、硬编码 Gemini price tuple、substring/default branch 或
`unwrap_or(0.0)` 时失败。

### 2. Exact Google pricing authority

在 `PricingService` 内建立唯一 exact Google lookup path。输入是 surface
(`GeminiDeveloper`/`VertexAi`)、requested model 与 `PricingUsage`：

1. 通过 merged GH1112 catalog exact-resolve canonical ID 并验证对应 surface availability；
2. 以明确的 pricing-provider mapping + canonical ID 查询当前已加载 pricing source；
3. 只接受 exact key 或 GH1112 显式 alias；Google path 禁止进入
   `is_shared_model_match`、longest-candidate、family substring 或另一 surface 的 availability；
4. 对 Gemini/Vertex token-priced record 始终验证 input/output token price 同时存在、
   finite 且非负；modal price 仅在对应 usage 非零时额外要求；
5. 返回现有 `PricingCostBreakdown`/`CostResult` typed result，保留 requested provider、
   resolved canonical model 与 pricing source identity 的可测试上下文。

custom `pricing.source` 是 live runtime authority；缺 record/field 必须 typed fail。neutral
Google catalog 中为兼容迁移而保留的 price metadata 不能在 live miss 时充当第二 fallback。
无法接收 runtime service 的 public compatibility helper 使用一个 fail-closed、lazy initialized
embedded `PricingService` adapter；初始化失败直接 typed error，不能构造空 service。live
gateway 不得调用该 embedded adapter。

错误边界使用一个 crate-private closed pricing-failure kind，并穷举映射到现有 public error
层级；不再依赖自由文本：

- exact authority 内部 failure kind 至少穷举 `UnknownModel`、`SurfaceUnavailable`、
  `MissingPrice`、`InvalidPrice`；普通 `GatewayError` 文本不能决定 policy eligibility；
- provider/public facade 穷举映射到独立结构化
  `ProviderError::PricingUnavailable { provider, model, reason }`（最终字段名可等价调整），
  `AllowUnpriced` 只按 variant 匹配，不得匹配 `message.starts_with(...)`；
- mapping 只包含 provider/requested+canonical model 和 field class，不包含 secret 或内容。

### 3. 单位与 public helper 收敛

authority 内部统一为 USD/token，conversion owner 只有一处：

```text
per_token = per_1k / 1_000
per_token = per_million / 1_000_000
total = input_tokens * input_per_token
      + output_tokens * output_per_token
      + other_declared_usage_units * their_per_unit_rates
```

每条来源记录必须携带可判定的 source unit；重复转换、heuristic unit detection 和 helper-local
`/1000`/`/1_000_000` 禁止。1、1_000、1_000_000 token 与 mixed/zero/large fixtures
用精确期望值和容差策略证明不发生 1_000 倍漂移。

scope inventory 中所有 helper 改为 typed `Result` 并薄委托 authority。若 helper 无法在不复制
pricing logic 的前提下保留，保留 symbol 但改为 typed adapter；不得另留 `_legacy`、
deprecated `Option` wrapper 或可达的 default calculator。`CHANGELOG.md` 列出签名迁移：

- `Option<f64>` / `f64` → `Result<f64, ProviderError>`（provider namespace）；
- already-typed trait/facade 保持签名，只修正 unknown/incomplete 的错误行为。

multimodal helper 以 `PricingUsage` 传递 cache/image/audio/video usage；缺对应 price field
typed fail，不得忽略 modality 或退回 basic token price。

### 4. Runtime parity 与 AllowUnpriced

reservation、settlement、spend 与 callback 继续使用 `AppState` 中同一个 runtime
`PricingService`，并传递实际 selected deployment 的 provider + canonical model。每次 retry
或 fallback 都新建 lookup；不能复用前一个 candidate 的 breakdown。

typed pricing-unavailable error 只在现有 gateway request-time policy boundary 被分类：

- `Reject`：记录 reject evidence 并在 provider call/budget mutation/success side effect 前返回；
- `AllowUnpriced`：仅配置明确选择该 variant 时，以
  `unpriced_fallback_cost_per_1k_tokens` 和同一 usage 计算 reservation/settlement；
- classifier 必须对 structured provider-error variant 做 closed match；authentication、network、
  serialization、budget、普通 `InvalidRequest`/`ModelNotFound` 或其他 error 不进入该 branch。

每次 bypass 继续调用 `record_unpriced_event`/`record_unpriced_spend`，输出结构化 `error`
log；有 API key 时写 `UsageRecord::unpriced`。provider、model、policy、outcome、usage units
与 fallback cost 必须在 reservation/settlement/record 间一致。fallback cost 为 0 仍标记
unpriced。记录失败必须 error；不引入新数据库表或吞掉已经完成的 provider response。

### 5. Side-effect、security 与 compatibility gates

失败矩阵使用 counters/loopback fixture 证明 unknown/wrong-surface/incomplete pricing 在
credential acquisition、network、budget reservation、priced spend、cache insert 与
successful callback 前停止。AllowUnpriced 的 bypass evidence 不记录 API key、Bearer、
project/location、prompt、response 或 raw model metadata。

GH1112 的 Gemini query-key 与 Vertex Bearer 路径只读，不在本 manifest 中修改。partner
model、image/TTS pricing 与非 Google provider resolver 回归必须保持不变。

## Product-to-Test Mapping

| Invariant | Implementation area | Verification |
| --- | --- | --- |
| B-001 | merged Google exact registry + exact `PricingService` Google resolver | exact Developer/Vertex surface matrix；unknown/fuzzy/prefix negatives；single-owner source guard |
| B-002 | authority validation + provider error mapping | unknown/retired/wrong-surface/missing/one-sided/NaN/negative price typed errors；one-sided/zero usage still validates both token fields；no zero/default success |
| B-003 | provider trait/facade + four public helper owners | compile-time signature fixtures；no `Option`/bare-f64/legacy calculator inventory |
| B-004 | single unit conversion owner | 1/1k/1M input/output/mixed/large numeric fixtures；no helper-local scaling guard |
| B-005 | helper, reservation, settlement, spend, callback | exact identity/source sentinel parity and zero-usage validation |
| B-006 | request preflight + retry/fallback | upstream/auth/network/budget/cache/callback counters zero；candidate-specific fresh lookup |
| B-007 | closed pricing-failure kind + structured provider-error variant + gateway classifier | `AllowUnpriced` matches only the structured pricing-unavailable variant；source guard rejects Display/message-prefix parsing；other errors remain errors |
| B-008 | unpriced reserve/settle/metrics/log/usage record | positive/zero fallback audit matrix；record failure emits error |
| B-009 | GatewayError→ProviderError mapping/redaction | exact variants/context fixtures plus sentinel secret/content capture |
| B-010 | GH1112 auth/catalog and partner paths | Google auth isolation + partner/image/TTS non-regression |

## 数据流

```text
requested provider/model + usage
  -> GH1112 exact canonical ID + surface availability
  -> runtime PricingService exact loaded-source lookup
  -> typed PricingCostBreakdown
       -> provider/public Result adapter
       -> budget reservation
       -> settlement + priced spend + callback

structured pricing-unavailable variant
  -> Reject: fail before upstream/mutation
  -> explicit AllowUnpriced only:
       fallback reserve -> fallback settle
       -> unpriced metric + structured error log
       -> UsageRecord::unpriced when key context exists
```

## 风险

- **Correctness**：Google price keys/provider aliases currently permit fuzzy cross-match；exact
  resolver must be isolated so unrelated providers retain compatibility while Google cannot drift。
- **Compatibility**：public `Option`/bare-`f64` helpers and the public `ProviderError` variant set
  change；migration note and compile
  fixtures are mandatory，but user has explicitly selected typed `Result` convergence。
- **Data**：GH1112 may move price metadata without refreshing values；implementation must use the
  loaded runtime source and may not invent missing prices。
- **Security**：provider/model and field class are safe audit dimensions；credentials、project/location
  and request/response content are prohibited from errors/logs/records。
- **Operations**：explicit AllowUnpriced can intentionally carry cost risk；every bypass therefore
  remains separately observable and is never labeled priced。
- **Overlap**：#1117 owns Google catalog shape；#1104/GH1103 owns broader pricing compatibility。
  GH1113 may consume the merged exact API and migrate only the enumerated Google cost helpers。

## 测试计划

- [ ] Dependency/manifest: confirm #1117 merge SHA, exact merged Google paths/symbols, fresh duplicate
  search, and reviewed manifest amendment if paths changed.
- [ ] Exact authority: Developer/Vertex availability matrix；canonical/prefixed/alias positives；
  unknown/fuzzy/retired/wrong-surface negatives；missing/one-sided/NaN/negative pricing；
  input-only/output-only/zero usage all reject one-sided token records.
- [ ] Units: 1、1_000、1_000_000 input/output；mixed、zero、large；cache/image/audio/video
  fields for multimodal helper；single conversion owner guard.
- [ ] Public API: compile fixtures for `LLMProvider`、`Provider` facade、Gemini inherent/basic/
  multimodal and Vertex Gemini/Vertex provider helpers returning typed `Result`; unknown never
  `Ok(0.0)` or Flash-priced.
- [ ] Runtime parity: custom-source sentinel across helper/runtime adapter、reservation、settlement、
  spend and callback；retry/fallback uses selected deployment identity.
- [ ] Policy/audit: default Reject and explicit AllowUnpriced positive/zero-fallback matrices；
  closed structured-variant classifier；ordinary `InvalidRequest` with the old
  `model_not_priced:` prefix and every non-pricing error remain ineligible；source guard proves
  no Display/message parsing；metrics/log/`UsageRecord::unpriced` fields and record-failure error capture.
- [ ] Security: Gemini key/Vertex Bearer loopback isolation and adversarial error/log/audit redaction.
- [ ] Regression: partner models、Vertex image generation/TTS、non-Google pricing resolvers.
- [ ] Coverage: new executable lines ≥80%；exact rejection、unit conversion、error mapping、
  AllowUnpriced classifier/audit critical branches 100%，using exact-head branch LCOV and fail-closed
  policy artifact.
- [ ] Repository: `cargo fmt --all -- --check`；
  `cargo check --all-targets --all-features --locked`；
  `cargo clippy --all-targets --all-features --locked -- -D warnings`；
  `cargo test --all-features --locked -- --test-threads=1`。

## 回滚方案

Implementation PR 可整体 revert 回到旧 helper，但旧 unknown-zero/default-Flash 行为只能作为
紧急回滚状态，不能被标记正确或 `AllowUnpriced`。本改动无 schema migration。若 public
signature rollout 需要分批，先保留同名 typed adapter 并在一个 release note 中迁移 callers；
不得用旧 `Option`/裸 `f64` wrapper 恢复第二 authority。
