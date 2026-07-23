# Tech Spec

## Linked Issue

GH-1103 / #1103

## Product Spec

Link to `product.md`.

## Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| Canonical runtime authority | `src/core/pricing_service/authority.rs`, `src/core/pricing_service/service.rs`, `src/core/pricing_service/types.rs` | provider-aware loaded-data lookup and cost calculation power user-visible pricing paths; `resolve_model_info_for_provider` reaches live provider catalogs through `provider_catalog_model_info` and dedicated Amazon Nova/xAI helpers | Must remain the single authority and replacement destination; every authority-reachable catalog must be inventoried |
| Compatibility facade | `src/core/cost/mod.rs`, `src/core/cost/calculator.rs`, `src/core/cost/types.rs`, `src/core/cost/utils.rs` | exposes public DTOs/trait/functions and maps authority results into legacy shapes | Public lifecycle and adapter boundary under review |
| Published pricing baseline | `v0.5.0@de594c81:src/core/cost/**`, `src/core/providers/base/{pricing.rs,mod.rs}`, `src/utils/{mod.rs,ai/**}` plus feature-gated provider modules | the released tag exposes `core::cost`, provider-base pricing, `utils::{ModelUtils,TokenUtils}` through multiple public paths, and optional provider cost/pricing APIs | All pricing-bearing paths found under default, no-default/lite, and docs.rs feature sets form the tag/package-derived 0.6 compatibility cohort |
| Published utility pricing | `v0.5.0@de594c81:src/utils/ai/models/pricing.rs`, `src/utils/ai/models/utils.rs`, `src/utils/ai/tokens.rs`, `src/utils/ai/mod.rs`, `src/utils/mod.rs` | `utils::ModelUtils::get_model_pricing` and `utils::TokenUtils::calculate_cost` (plus nested module paths) contain independent hard-coded lookup/calculation logic | Signature-preserving adapters still need compatibility and authority disposition; top-level struct inventory alone is insufficient |
| Published feature matrix | `v0.5.0@de594c81:Cargo.toml`, `README.md`, feature-gated `src/core/providers/**` | defaults are `sqlite,redis,metrics,tracing`; README documents API-only `--no-default-features --features lite`; docs.rs adds `gateway,postgres,s3,websockets,analytics,providers-extra,providers-extended` and exposes pricing APIs such as Azure re-exports and Bedrock `CostCalculator` | Baseline extraction and fixtures must run all three exact feature lanes |
| Post-v0.5 public pricing facade | `src/core/pricing.rs`, `src/core/mod.rs` | introduced by `04c0774a` after `v0.5.0`; current public `PricingDatabase`, `GLOBAL_PRICING_DB`, `get_pricing_db`, `calculate_cost` and related methods can load/lookup/calculate independently, including legacy `0.0` misses | Must receive current-head authority disposition, but is not a v0.5 published import/signature baseline |
| Legacy fallback catalogs | `src/core/cost/calculator/pricing.rs`, `src/core/cost/calculator/pricing/**`, `src/core/cost/providers/**` | compatibility paths may use provider-specific catalog logic | Each fallback needs evidence-backed disposition |
| Live authority fallback catalogs | `src/core/pricing_service/authority.rs::provider_catalog_model_info`, `src/core/cost/calculator/pricing.rs::get_azure_pricing`, `src/core/providers/bedrock/utils/cost.rs::{MODEL_PRICING,CostCalculator}`, `src/core/providers/registry/catalog.rs::amazon_nova_catalog_model_info`, `src/core/providers/openai_like/models.rs::{is_xai_priced_model,get_openai_like_registry}` | `PricingService` currently reads these Azure、Bedrock、Amazon Nova 与 xAI sources on loaded-data misses | Guard must trace every branch/source; provider-owned location does not exempt a live authority input |
| Duplicate result shapes | `src/core/cost/types.rs:368`, `src/core/pricing_service/types.rs:55` | two `CostResult` types serve compatibility and authority layers | Same name does not prove safe deletion; conversion/consumer inventory is required |
| Public exports | `src/core/mod.rs`, `src/core/cost/mod.rs`, `src/core/pricing.rs`, `src/core/pricing_service/mod.rs` | all three pricing/cost modules expose public symbols | Downstream library imports create semver risk even when gateway runtime is unchanged |
| Direct provider cost API | `src/core/providers/mod.rs::Provider::calculate_cost`, `src/core/traits/provider/llm_provider/trait_definition.rs::LLMProvider::calculate_cost` | the public enum facade strips a provider prefix and dispatches directly to public trait implementations, whose hand-written and macro-generated bodies can retain provider-local pricing behavior | Direct SDK/library callers can bypass the inventoried route and compatibility facades unless the facade, trait, and every implementation receive an explicit authority/compatibility disposition |
| Live consumers and unpriced policy | `src/server/routes/ai/spend.rs`, `src/server/routes/ai/spend/unpriced.rs`, `src/server/routes/ai/callbacks.rs`, `src/server/routes/ai/response_cache.rs`, chat/embedding routes, provider modules importing `core::cost`, pricing routes | GH726 routes priced calculation through authority-backed helpers; callback terminal events calculate cost through an `Arc<PricingService>`; chat/embedding cache admission checks query `state.budgeted.pricing()`; default `Reject` fails closed, while explicit `AllowUnpriced` reserves and settles configured fallback cost | Tests must distinguish authority calls, compatibility use, callback/cache consumers, and intentional policy fallback |
| Pricing source boundary | `src/server/http.rs`, `src/core/pricing_service/authority.rs`, `src/core/cost/calculator.rs`, provider-base/utility adapters | live state constructs `PricingService` from configured `pricing.source`; v0.5-signature compatibility adapters have no service parameter and use embedded data | Parity must compare consumers using the same loaded source; cross-source numeric equality is not a valid invariant |
| Predecessor packet | `specs/GH726/*` | deliberately retained legacy DTO/fallback compatibility after runtime convergence | This issue may narrow that deferral, not rewrite GH726 |

## Planned Changes

```specrail-planned-changes
{
  "issue": 1103,
  "complete": true,
  "paths": [
    "src/core/cost/**",
    "src/core/pricing.rs",
    "src/core/pricing_service/**",
    "src/core/providers/base/mod.rs",
    "src/core/providers/base/pricing.rs",
    "src/core/providers/mod.rs",
    "src/core/traits/provider/llm_provider/trait_definition.rs",
    "src/utils/mod.rs",
    "src/utils/ai/**",
    "src/server/routes/ai/spend.rs",
    "src/server/routes/ai/spend/unpriced.rs",
    "src/server/routes/ai/callbacks.rs",
    "src/server/routes/ai/response_cache.rs",
    "src/server/routes/pricing.rs",
    "CHANGELOG.md"
  ],
  "spec_refs": ["P1", "P2", "P3", "P4", "P5", "P6", "P7"]
}
```

Provider-specific hand-written and macro-generated `LLMProvider::calculate_cost` implementations are inventory
targets reached from the two declared public owner paths, not blanket writable provider scope. T1 must enumerate
their exact paths before T2; any implementation file needed by an approved T2 disposition must be added explicitly
to this manifest and re-reviewed before T3 writes it.

## 设计方案

按四个有序阶段执行；本 packet PR 只完成设计和任务拆分。

### Phase 1 — inventory 与守护

- 生成 tracked inventory，按完整 Rust path 分开记录：(a) `v0.5.0@de594c81` 实际发布的 `core::cost`、
  `core::providers::base::pricing`、`utils::ModelUtils::get_model_pricing`、`utils::TokenUtils::calculate_cost` 及其
  `utils::ai`/deeper module re-export path，并遍历 default、no-default/lite、docs.rs feature matrix 下的 provider
  public pricing/cost API；
  (b) post-v0.5/current-head `core::pricing` public export、production consumer、test-only consumer、DTO conversion、
  公开 lookup/calculation method、`Provider::calculate_cost` enum dispatch、
  `LLMProvider::calculate_cost` trait declaration及其全部手写/宏生成 implementation 与 fallback owner。
- live fallback inventory 必须以 `pricing_service/authority.rs::resolve_model_info_for_provider` 调用图为根，至少锁定
  `provider_catalog_model_info` 的 Azure/Bedrock/xAI branches、`amazon_nova_pricing_model_info`、
  `xai_pricing_model_info`，并追踪到 `core::cost::calculator::pricing`、Bedrock `CostCalculator` catalog、
  `providers::registry::catalog::amazon_nova_catalog_model_info` 与 `providers::openai_like::models`；新增 branch、helper
  或 authority-reachable catalog 未登记时 guard 必须失败。
- 从 `v0.5.0@de594c81` tag 与已发布 package surface 生成 public API baseline manifest；manifest 必须证明
  `src/core/pricing.rs` 在 tag 中不存在，并把当时的 `core::providers::base::pricing` module/re-export 与
  `core::cost`、utility pricing methods 以及 feature-gated provider pricing/cost path 完整列为 published cohort。
  baseline generator 必须分别在 default features、`--no-default-features --features lite` 与 v0.5 docs.rs exact
  feature set
  `gateway,postgres,sqlite,redis,s3,metrics,tracing,websockets,analytics,providers-extra,providers-extended` 运行并合并
  full-path manifest；当前树 inventory 必须说明这些 symbol 是保留、重导出还是进入 deprecation，不能用
  current-head module path 或默认 feature scan 倒推全部已发布 API。
- v0.5 published adapter disposition 只能是 `keep_adapter`、`deprecate_0_6_remove_0_7` 或 `needs_decision`。
  post-v0.5 `core::pricing` 先记录 `post_v0_5_unreleased` baseline status，其中可独立 load/lookup/calculate 的
  authority-bearing public facade 仍必须有 `migrate_authority` 或 `needs_decision` authority disposition，但不自动
  获得 `deprecate_0_6_remove_0_7` 状态。user-visible fallback disposition同样只能是 `migrate_authority` 或
  `needs_decision`。任何独立 lookup/calculation authority 都不得仅以 `keep_adapter` 保留。
- source guard 必须在新增未登记 public export、production consumer 或 fallback 时失败，并把
  `CallbackLifecycle` terminal-cost 与 chat/embedding response-cache pricing gate 固定为 live consumer roots；
  `Provider::calculate_cost`、`LLMProvider::calculate_cost` 或任一 provider implementation 未登记也必须失败；
  不得依赖字符串命中自动判定某项可删除。

### Phase 2 — 0.6 compatibility/deprecation

- `migrate_authority` 项把价格解析/匹配逻辑移到 `PricingService` authority 后，再由 `core::cost` 或批准保留的
  `core::pricing`/utility adapter 映射 legacy DTO/error/tuple；所有 user-visible fallback lookup/calculation 都必须在
  authority 内运行，compatibility facade 不得复制或保留 catalog lookup 形成第二套 authority。0.6 对 v0.5
  published `core::cost`/`core::providers::base::pricing`/utility/feature-gated provider API 的签名不得静默改签名；
  legacy miss 行为必须在 compatibility matrix 中逐 symbol 记录，且 gateway production 不得使用该行为绕过 pricing policy。
- 仅对 v0.5 published cohort 中批准为 `deprecate_0_6_remove_0_7` 的 public symbol 添加
  `#[deprecated(since = "0.6.0", ...)]` 并保持签名、结果与 error contract。post-v0.5/current-head `core::pricing`
  可在 0.6 发布前按 T2 authority decision 迁移或收敛，不要求伪造 v0.5 import fixture，也不自动推迟到 0.7；
  若某 symbol 被有意发布进 0.6，后续 removal 再受该 release 的 public compatibility policy 约束。
- 用 `v0.5.0` baseline manifest 和下游式 compile/behavior fixture 证明每个已发布 `core::cost`、
  `core::providers::base::pricing`、utility pricing 与 feature-gated provider import 在 0.6 head 仍可用。fixture
  必须有 default-features lane、`--no-default-features --features lite` API-only lane 与上述 v0.5 docs.rs
  exact-feature lane；lite lane 必须实际 import/call其可用的 public pricing surface，docs.rs lane 至少实际
  import/call Azure pricing re-export 与 Bedrock `CostCalculator::{get_model_pricing,calculate_cost}`，不能只证明
  feature 可编译。
  另用 current-head authority fixture 检查 `core::pricing` disposition，不把它混入 tag-derived fixture。再覆盖
  source-aware authority/facade parity、provider alias/fallback、默认 `Reject` policy 的 unknown/incomplete pricing fail-closed，
  以及显式 `AllowUnpriced` policy 按配置 fallback cost 的 reservation/settlement parity，并同步 CHANGELOG 与迁移说明。
- authority/facade parity 必须绑定 source identity：(a) custom-source fixture 用带 sentinel price 的临时 source 构造
  runtime `PricingService`，证明 pricing route、reservation、settlement、callback terminal cost 与
  chat/embedding response-cache admission pricing gate 全部读取同一 loaded authority；cache hit/miss 均不得绕过
  pricing gate；(b) embedded
  fixture 比较 v0.5-signature `core::cost`/provider-base/utility facade 与
  `PricingService::with_embedded_default()`。禁止比较 custom
  与 embedded 的数值，或假设 v0.5 facade 已获得 service injection；未来 non-breaking injection 需要独立批准。

### Phase 3 — release 与 version workflow gate

- 记录包含 targeted deprecation 的已验证 0.6.x release artifact。
- version workflow 用 deterministic fixture 证明从 0.6.x breaking change 得到 0.7.0，而不是 1.0.0、patch
  或非 breaking label。
- public API owner 明确批准最终 removal 清单；`needs_decision` 项不得进入 removal。

### Phase 4 — 0.7 removal

- 只删除 Phase 2 已发布 deprecated 且 Phase 3 清单批准的 symbol/adapter/fallback。
- post-v0.5/current-head `core::pricing` 不因被 inventory 覆盖而自动进入本阶段；其发布前 authority 收敛属于
  Phase 2，只有实际进入 0.6 release 的 public symbol 才对后续 breaking removal 产生新的 compatibility gate。
- 删除后继续保留 `PricingService` authority、endpoint/spend semantics 与不在清单内的 provider-local catalog。
- 运行 public removal fixture、authority/facade regression、全量测试与 closure audit；任何 scope expansion 另开 spec。

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P1 | runtime configured `PricingService`, embedded compatibility adapters, `spend/unpriced.rs`, spend/pricing routes, callback lifecycle, chat/embedding response-cache pricing gates | custom-source route/reserve/settle/callback/cache parity and separate embedded facade parity; default `Reject` fail-closed; `AllowUnpriced` reserve/settle parity |
| P2 | `v0.5.0@de594c81` API manifest for core/provider-base/utility/feature-gated pricing exports；separate current-head `core::pricing`、`Provider::calculate_cost` 与 `LLMProvider::calculate_cost` authority inventory | default + no-default/lite + exact docs.rs feature downstream fixtures for the published cohort; current-head direct-call authority/error fixtures for `core::pricing`, Provider facade and LLMProvider implementations |
| P3 | `provider_catalog_model_info`, Amazon Nova/xAI helpers, Azure/Bedrock/provider-owned catalogs and compatibility fallbacks | exhaustive call-graph/disposition guard plus provider alias/fallback tests |
| P4 | release/version workflow and approved removal list | deterministic 0.6.x → 0.7.0 fixture and public removal compile fixture |
| P5 | pricing usage/result conversion, callback lifecycle, cache admission and provider regressions | cached/reasoning/multimodal/time-based focused tests plus callback terminal-cost and chat/embedding cache hit/miss pricing-gate source sentinels |
| P6 | routes, persistence and network boundaries | exact-diff scope guard and existing route/spend tests |
| P7 | #519, #729, #965 references | roadmap reconciliation review |

## 数据流

调用输入仍是 provider、model、usage 与可选 modality/duration metadata。live `PricingService` 从配置的
`pricing.source` 加载数据并执行 inventory 已登记或批准迁入的 provider fallback 后返回 canonical result；route、
reservation、settlement、callback terminal cost 与 chat/embedding cache admission 共享该 runtime
instance/source。无 service 参数的 v0.5-signature compatibility adapter
使用 embedded authority，并只把该同源 canonical result/error 转成逐 symbol 批准的 legacy contract；不得用它
代表 custom source。authority miss 后 adapter 不得运行独立 catalog lookup。live gateway 在默认 `Reject` policy
下返回显式 not-found/incomplete-pricing error；只有显式
`AllowUnpriced` policy 可按 `unpriced_fallback_cost_per_1k_tokens` 对同一 usage 执行 fallback reservation 与
settlement，不得把该 policy 泛化成隐式零成本成功。
本工作不新增持久化、外部请求、后台任务或路由。

## 备选方案

- 立即删除整个 `core::cost`：拒绝；公开 import、provider consumer 与 0.6 兼容窗口未完成。
- 永久保留所有 duplicate DTO/fallback：拒绝；会让 #519 A-6 永久没有 lifecycle owner。
- 把所有 provider-local catalog 迁入一个 PR：拒绝；范围不可审查，并与 #837/provider-specific ownership 冲突。
- 重新实现 #726 authority convergence：拒绝；live authority 已完成，本 issue 只处理明确延期的兼容面。

## 风险

- Security: 不处理 secrets/auth；但错误 fallback 可能低估成本，必须保持 fail-closed。
- Compatibility: v0.5 published Rust import removal 是 breaking change；0.6 对 `core::cost`、
  `core::providers::base::pricing`、utility pricing methods 与 feature-gated provider pricing path 的 deprecation、
  迁移文档与 human approval 为硬门禁。post-v0.5
  `core::pricing` 的 authority 收敛仍需 T2 决策，但不能被误报成 v0.5 signature break。
- Correctness: custom source 与 embedded source 可合法产生不同结果；测试必须绑定 source identity，不能以跨 source
  parity 假阳性/假阴性驱动签名变更。
- Performance: adapter 不得每次重新解析 bundled pricing；继续复用已加载或 `LazyLock` authority。
- Maintenance: inventory guard 需要完整 path、authority call-graph root 与显式 disposition，避免同名 DTO 或
  provider-owned catalog 因目录位置被误判为无关。
- Overlap: `src/core/providers/**` 与 #837、router/registry 与 #965 均为默认禁止写入范围，除非后续 spec 明确缩小并重新 gate。

## 测试计划

- [ ] Inventory guard: v0.5 `core::cost`/`core::providers::base::pricing`、utility pricing method、
  default/no-default-lite/docs.rs
  feature-gated published export，以及 post-v0.5 `core::pricing` current-head export/production consumer、
  `Provider::calculate_cost`/`LLMProvider::calculate_cost` declaration、dispatch 与全部 implementation、
  `provider_catalog_model_info`/Amazon Nova/xAI helper、callback lifecycle、chat/embedding cache pricing gate 与
  全部 authority-reachable/legacy fallback 均有 disposition，
  新增 branch、decoy 与漏项负测试失败。
- [ ] Unit tests: authority-to-legacy DTO conversion、默认 `Reject` 的 unknown/incomplete pricing fail-closed、
  `AllowUnpriced` configured fallback、provider alias/fallback（Azure、Bedrock、Amazon Nova、xAI）。
- [ ] Integration tests: custom-source live pricing route/reservation/settlement/callback terminal cost/chat+embedding
  cache admission gate parity，且 cache hit/miss 均不绕过 pricing；embedded authority 与
  v0.5-signature facade parity；Provider enum facade/LLMProvider direct-call fixtures 对每个批准 source/error
  contract 有相同 disposition；`AllowUnpriced` 对同一未知 usage 的 reservation/settlement/usage-record cost parity；
  不含跨 source equality assertion。
- [ ] Compatibility: `v0.5.0@de594c81` tag/package-derived core/provider-base/utility/feature-gated public API manifest；
  default、no-default/lite 与 exact docs.rs feature 0.6 downstream import/legacy behavior fixtures；单独的 post-v0.5/current-head
  `core::pricing` authority disposition fixture；0.7 只对 approved published-cohort removal 运行 compile-fail/替代 import fixture。
- [ ] Version workflow: deterministic 0.6.x breaking fixture 产出 0.7.0。
- [ ] Repository: `cargo fmt --all -- --check`; `cargo check --all-targets --all-features --locked`; `cargo clippy --all-targets --all-features --locked -- -D warnings`; `cargo test --all-features --locked -- --test-threads=1`。

## 回滚方案

0.6 tranche 可整体 revert，恢复未标记 deprecated 的 compatibility exports；authority migration 必须与 adapter 变更一起回滚，
不能留下第二套 source。0.7 removal 通过独立 breaking PR 交付，可在未发布前 revert；发布后按 migration note 恢复 compatibility
adapter 需要新的 semver 决策。没有数据库迁移或数据回滚。
