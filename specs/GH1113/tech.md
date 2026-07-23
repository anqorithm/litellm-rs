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
| Gemini module and utility pricing lookups | `src/core/providers/gemini/mod.rs:63-70`, `src/utils/ai/models/pricing.rs:3-68,200-220` | Public module lookup returns `Option<(f64, f64)>` from the Gemini registry; `ModelUtils` lowercases/fuzzy-matches and carries hard-coded Gemini tuples. | These are public duplicate Google pricing paths and require a typed/exact disposition. |
| Core cost compatibility facade | `src/core/cost/calculator.rs:52-125`, `src/core/cost/calculator/pricing.rs:484-510` | The embedded `PricingService` falls back to an empty service on initialization failure, then Gemini/Vertex can fall through to a hard-coded family table. | Compatibility callers can bypass the canonical authority unless initialization and Google fallback both fail closed. |
| Shared pricing database and HTTP API | `src/core/pricing.rs:192-249,678-682`, `src/server/routes/pricing.rs:121-200` | Public database methods use normalized/fuzzy lookup and return `0.0` on miss; `/v1/pricing` exposes model lookup and calculation without a Google surface contract. | User-facing Google pricing must not retain fuzzy/zero/ambiguous-surface behavior. |
| Canonical pricing authority | `src/core/pricing_service/authority.rs:24-190,193-247,303-399,421-490`, `src/core/pricing_service/service.rs:108-183` | Loaded provider-aware APIs return `Result`, but shared resolver still permits alias/fuzzy matching and converts provider-local per-1k values. | GH1113 must add an exact Google path without changing unrelated provider compatibility. |
| Providerless service and usage carrier | `src/core/pricing_service/service.rs:101-155`, `src/core/pricing_service/types.rs:74-89` | Public providerless Google APIs lack surface input; `PricingUsage` lacks video seconds. | These owners need typed surface disposition and lossless video transport. |
| Runtime reservation/settlement | `src/server/routes/ai/spend/pricing.rs`, `src/server/routes/ai/spend/completion.rs`, `src/server/routes/ai/chat_streaming.rs`, `src/server/routes/ai/spend.rs` | Live paths use runtime `PricingService`, convert misses to model-not-priced errors, and share budget/spend identities. | Required parity and pre-upstream fail-closed evidence. |
| Image generation and terminal callbacks | `src/server/routes/ai/images/generation.rs:100-149`, `src/server/routes/ai/callbacks.rs:222-275` | Image generation reserves/settles through policy helpers; callbacks independently recalculate terminal cost and drop it on pricing error. | Typed policy changes must cover the normal image route, and callback cost must match priced or explicit-unpriced settlement. |
| Terminal identity callsites | `src/server/routes/ai/chat.rs:267`, `src/server/routes/ai/completions_streaming.rs:315`, `src/server/routes/ai/gemini/provider.rs:19-24`, `src/server/routes/ai/gemini.rs:163-178,243-255` | Callback callsites do not receive settled cost; Gemini hard-codes requested identity. | B-005 requires callsite ownership to thread terminal cost and selected pricing identity. |
| Facade/error transport | `src/core/providers/mod.rs:562-586`, `src/core/providers/unified_provider_http_mapping.rs:84-89,308-309` | Facade strips prefix before Google validation; HTTP mapping parses message prefix. | Actual owners are required for Google-only prefix validation and structured Reject mapping. |
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
    "src/core/pricing_service/mod.rs",
    "src/core/pricing_service/service.rs",
    "src/core/pricing_service/service_tests.rs",
    "src/core/pricing_service/types.rs",
    "src/core/pricing.rs",
    "src/core/pricing/tests.rs",
    "src/core/cost/calculator.rs",
    "src/core/cost/calculator/pricing.rs",
    "src/core/cost/calculator/tests/edge_case_tests.rs",
    "src/core/cost/calculator/tests/pricing_lookup_tests.rs",
    "src/core/providers/gemini/mod.rs",
    "src/core/providers/gemini/models/mod.rs",
    "src/core/providers/gemini/provider.rs",
    "src/core/providers/gemini/provider_tests.rs",
    "src/core/providers/mod.rs",
    "src/core/providers/unified_provider_http_mapping.rs",
    "src/core/providers/vertex_ai/client.rs",
    "src/core/providers/vertex_ai/client_tests.rs",
    "src/core/providers/vertex_ai/gemini/mod.rs",
    "src/core/providers/vertex_ai/tests.rs",
    "src/utils/ai/models/pricing.rs",
    "src/server/routes/pricing.rs",
    "src/server/routes/ai/callbacks.rs",
    "src/server/routes/ai/chat.rs",
    "src/server/routes/ai/completions_streaming.rs",
    "src/server/routes/ai/spend.rs",
    "src/server/routes/ai/spend/pricing.rs",
    "src/server/routes/ai/spend/completion.rs",
    "src/server/routes/ai/spend/unpriced.rs",
    "src/server/routes/ai/gemini/spend.rs",
    "src/server/routes/ai/gemini/provider.rs",
    "src/server/routes/ai/gemini.rs",
    "src/server/routes/ai/audio/budgeting.rs",
    "src/server/routes/ai/images/generation.rs",
    "src/server/routes/ai/images/proxy_spend.rs",
    "src/server/routes/ai/response_cache.rs",
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

`src/core/providers/google/models/**` 仅列作 dependency anchor，不是 writable manifest path；
本 issue 对 merged GH1112/GH1108 catalog 及 price metadata 只读。duplicate-calculator guard
仅豁免这些 metadata 定义，并须证明其对 live miss fallback/user-visible authority 不可达；
不得编辑、删除或刷新。若 #1117 合并结果改名、拆分或缺少本 spec
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
- `gemini::get_model_pricing` 与 `ModelUtils::get_model_pricing`；
- `core::cost::calculator::{generic_cost_per_token, get_model_pricing}`；
- `PricingDatabase::{calculate, calculate_for_provider}` 与全局
  `core::pricing::calculate_cost` 的 Google 分支；
- `/v1/pricing/model/{model_name}` 与 `/v1/pricing/calculate` 的 Google 请求；
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

public providerless `PricingService::{get_model_info, calculate_completion_cost}` 的 Google 输入
必须接收/推导明确 Developer/Vertex surface，否则 typed fail；不得从 record provider 猜测。
`PricingUsage` 增加 video usage carrier，authority 统一验证与计价，禁止 helper-local fallback。

错误边界使用一个 crate-private closed `PricingFailureKind`，由 `pricing_service::mod` 仅在
crate 内提供给 request-time policy；public error mapping 发生在 policy eligibility 决策之后，
不再依赖自由文本：

- exact authority 内部 failure kind 至少穷举 `UnknownModel`、`SurfaceUnavailable`、
  `MissingPrice`、`InvalidPrice`；runtime reservation/preflight 直接传递该 typed fact，
  普通 `GatewayError`/`ProviderError` 文本不能决定 policy eligibility；
- public provider/helper 在 policy boundary 外穷举映射到现有 variants：
  `UnknownModel`/`SurfaceUnavailable` → `ProviderError::ModelNotFound`，
  `MissingPrice`/`InvalidPrice` → `ProviderError::Configuration`；
- default Reject 可在 eligibility 决策后映射为现有
  `ProviderError::InvalidRequest { provider: "pricing", ... }`。retry/fallback 若需识别该
  Reject 终态，只能匹配 enum variant + reserved `"pricing"` provider field；禁止读取 message。
  source guard 保证该 reserved construction 只有一个 owner；
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

`gemini::get_model_pricing`、`ModelUtils::get_model_pricing`、`PricingDatabase` 和
`core::pricing::calculate_cost` 对非 Google provider 保持兼容；一旦输入声明或解析为
Gemini/Vertex，则必须要求可判定的 surface/provider 并走 exact typed authority。无法区分
Developer 与 Vertex surface 的 Google 请求显式失败，不得猜测 Vertex、跨 surface 或回退
到无 provider fuzzy lookup。`/v1/pricing` 对这些 typed failures 返回稳定的非 2xx error
envelope；不得以 miss=`0.0` 或 model lookup success 替代错误。

`Provider::calculate_cost` 只在 Google 分支保留 requested prefix 交给 GH1112 exact validation；
所有非 Google provider 继续使用现有 prefix stripping 语义。

multimodal helper 以 `PricingUsage` 传递 cache/image/audio/video usage；缺对应 price field
typed fail，不得忽略 modality 或退回 basic token price。

### 4. Runtime parity 与 AllowUnpriced

reservation、settlement、spend、normal image generation 与 callback 继续使用 `AppState` 中同一个 runtime
`PricingService`，并传递实际 selected deployment 的 provider + canonical model。每次 retry
或 fallback 都新建 lookup；不能复用前一个 candidate 的 breakdown。

callback 不再独立重新计算并在失败时丢弃 cost；它消费与 terminal settlement 相同的
priced breakdown，或显式 `AllowUnpriced` 已决定并审计的 fallback cost。normal image
generation 与 proxy image route 均适配同一个 typed policy input，正负矩阵覆盖二者。
chat unary/streaming 从 settlement result 向 callback 传递同一 cost；Gemini unary/streaming
使用 router 实际 selected provider/model/deployment 构造 pricing identity，不改变 selection、
fallback order 或 endpoint shape。

crate-private `PricingFailureKind` 只在现有 gateway request-time policy boundary 被分类：

- `Reject`：记录 reject evidence 并在 provider call/budget mutation/success side effect 前返回；
- `AllowUnpriced`：仅配置明确选择该 variant 时，以
  `unpriced_fallback_cost_per_1k_tokens` 和同一 usage 计算 reservation/settlement；
- policy functions 必须接收 internal typed fact，不接受任意 `Display`、`GatewayError` 或
  `ProviderError`；authentication、network、serialization、budget、普通
  `InvalidRequest`/`ModelNotFound` 或其他 error 不进入该 branch。为保持全局
  `AllowUnpriced` 行为，现有 completion/usage/Gemini/audio/image pricing callsites 机械适配
  该 typed input；不得改变其价格或 policy 结果。

`unified_provider_http_mapping` 仅按 `InvalidRequest` variant + reserved `"pricing"` provider
field 产生 model-not-priced HTTP code；删除 message-prefix classifier，普通 InvalidRequest
仍映射为普通 invalid request。

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
| B-003 | provider trait/facade + Gemini module/utility + core cost + shared pricing DB + HTTP pricing owners | compile-time signature and route fixtures；no Google `Option`/bare-f64/fuzzy-zero/legacy calculator inventory |
| B-004 | single unit conversion owner | 1/1k/1M input/output/mixed/large numeric fixtures；no helper-local scaling guard |
| B-005 | helper, pricing API, image generation, reservation, settlement, spend, callback | exact identity/source/fallback sentinel parity and zero-usage validation |
| B-006 | request preflight + retry/fallback | upstream/auth/network/budget/cache/callback counters zero；candidate-specific fresh lookup |
| B-007 | crate-private pricing-failure kind + gateway policy boundary | `AllowUnpriced` receives only the internal typed fact before public-error mapping；source guard rejects Display/message-prefix parsing and public enum expansion；other errors remain errors |
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
       -> /v1/pricing typed response
       -> budget reservation
       -> image/runtime settlement + priced spend + callback

crate-private PricingFailureKind
  -> Reject: fail before upstream/mutation
  -> explicit AllowUnpriced only:
       fallback reserve -> fallback settle
       -> unpriced metric + structured error log
       -> UsageRecord::unpriced when key context exists
```

## 风险

- **Correctness**：Google price keys/provider aliases currently permit fuzzy cross-match；exact
  resolver must be isolated so unrelated providers retain compatibility while Google cannot drift。
- **Compatibility**：public `Option`/bare-`f64` helper signatures change as approved；the exhaustive
  public `ProviderError` variant set must not change。migration note and compile fixtures are mandatory。
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
  fields 经 `PricingUsage` video carrier；single conversion owner guard.
- [ ] Public API: compile fixtures for `LLMProvider`、`Provider` facade、Gemini inherent/basic/
  multimodal/module/ModelUtils、Vertex Gemini/Vertex provider、`core::cost`、shared
  `PricingDatabase` Google branches returning typed `Result`; `/v1/pricing` exact-surface
  route fixtures；providerless surface matrix；Google prefix validation 与 non-Google prefix
  compatibility；unknown never `Ok(0.0)`、HTTP success-zero or Flash-priced.
- [ ] Runtime parity: custom-source sentinel across helper/runtime adapter、reservation、settlement、
  normal/proxy image generation、spend and callback；explicit-unpriced callback preserves fallback
  cost；chat unary/streaming threads settled cost；Gemini unary/streaming 与 retry/fallback uses
  selected deployment identity.
- [ ] Policy/audit: default Reject and explicit AllowUnpriced positive/zero-fallback matrices；
  internal typed-fact classifier before public-error mapping；ordinary `InvalidRequest` with the old
  `model_not_priced:` prefix and every non-pricing error remain ineligible；Reject retry recognition
  matches only existing variant + reserved provider field；source guard proves no Display/message
  parsing or new public enum variant；metrics/log/`UsageRecord::unpriced` fields and record-failure
  error capture；HTTP mapping reserved-provider positive 与 ordinary-prefix negative。
- [ ] Catalog boundary: GH1112/GH1108 price metadata byte-for-byte unchanged；guard 只豁免定义且
  call graph 证明其不能作为 live miss fallback 或 user-visible cost。
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
分项回滚必须成组回滚 authority 与 service/video carrier、Google facade prefix branch、
HTTP structured mapping、chat/Gemini identity threading及 callers；不得留下混合状态。
catalog metadata 始终只读且不属于回滚 diff；非 Google prefix 语义不得改变。
