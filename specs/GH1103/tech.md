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
| Public pricing facade | `src/core/pricing.rs`, `src/core/mod.rs` | public `PricingDatabase`, `GLOBAL_PRICING_DB`, `get_pricing_db`, `calculate_cost` and related methods can load/lookup/calculate independently, including legacy `0.0` misses | Public compatibility and authority lifecycle must be dispositioned alongside `core::cost` |
| Legacy fallback catalogs | `src/core/cost/calculator/pricing.rs`, `src/core/cost/calculator/pricing/**`, `src/core/cost/providers/**` | compatibility paths may use provider-specific catalog logic | Each fallback needs evidence-backed disposition |
| Live authority fallback catalogs | `src/core/pricing_service/authority.rs::provider_catalog_model_info`, `src/core/cost/calculator/pricing.rs::get_azure_pricing`, `src/core/providers/bedrock/utils/cost.rs::{MODEL_PRICING,CostCalculator}`, `src/core/providers/registry/catalog.rs::amazon_nova_catalog_model_info`, `src/core/providers/openai_like/models.rs::{is_xai_priced_model,get_openai_like_registry}` | `PricingService` currently reads these Azure、Bedrock、Amazon Nova 与 xAI sources on loaded-data misses | Guard must trace every branch/source; provider-owned location does not exempt a live authority input |
| Duplicate result shapes | `src/core/cost/types.rs:368`, `src/core/pricing_service/types.rs:55` | two `CostResult` types serve compatibility and authority layers | Same name does not prove safe deletion; conversion/consumer inventory is required |
| Public exports | `src/core/mod.rs`, `src/core/cost/mod.rs`, `src/core/pricing.rs`, `src/core/pricing_service/mod.rs` | all three pricing/cost modules expose public symbols | Downstream library imports create semver risk even when gateway runtime is unchanged |
| Live consumers and unpriced policy | `src/server/routes/ai/spend.rs`, `src/server/routes/ai/spend/unpriced.rs`, provider modules importing `core::cost`, pricing routes | GH726 routes priced calculation through authority-backed helpers; default `Reject` fails closed, while explicit `AllowUnpriced` reserves and settles configured fallback cost | Tests must distinguish authority calls, compatibility use, and intentional policy fallback |
| Predecessor packet | `specs/GH726/*` | deliberately retained legacy DTO/fallback compatibility after runtime convergence | This issue may narrow that deferral, not rewrite GH726 |

## 设计方案

按四个有序阶段执行；本 packet PR 只完成设计和任务拆分。

### Phase 1 — inventory 与守护

- 生成 tracked inventory，按完整 Rust path 记录 `core::cost` 与 `core::pricing` public re-export、production consumer、
  test-only consumer、DTO conversion、公开 lookup/calculation method 与 fallback owner。
- live fallback inventory 必须以 `pricing_service/authority.rs::resolve_model_info_for_provider` 调用图为根，至少锁定
  `provider_catalog_model_info` 的 Azure/Bedrock/xAI branches、`amazon_nova_pricing_model_info`、
  `xai_pricing_model_info`，并追踪到 `core::cost::calculator::pricing`、Bedrock `CostCalculator` catalog、
  `providers::registry::catalog::amazon_nova_catalog_model_info` 与 `providers::openai_like::models`；新增 branch、helper
  或 authority-reachable catalog 未登记时 guard 必须失败。
- 从 `v0.5.0` tag 与已发布 package surface 生成 public API baseline manifest；当前树 inventory 必须显式说明
  baseline 中每个 symbol 是保留、重导出还是进入 deprecation，不能只扫描当前 head。
- public adapter disposition 只能是 `keep_adapter`、`deprecate_0_6_remove_0_7` 或 `needs_decision`；
  `core::pricing` 中可独立 load/lookup/calculate 的 authority-bearing public facade 还必须有 `migrate_authority`
  或 `needs_decision` authority disposition。user-visible fallback disposition同样只能是 `migrate_authority` 或
  `needs_decision`。任何独立 lookup/calculation authority 都不得仅以 `keep_adapter` 保留。
- source guard 必须在新增未登记 public export、production consumer 或 fallback 时失败；不得依赖字符串命中
  自动判定某项可删除。

### Phase 2 — 0.6 compatibility/deprecation

- `migrate_authority` 项把价格解析/匹配逻辑移到 `PricingService` authority 后，再由 `core::cost` 或批准保留的
  `core::pricing` adapter 映射 legacy DTO/error；所有 user-visible fallback lookup/calculation 都必须在 authority
  内运行，compatibility facade 不得复制或保留 catalog lookup 形成第二套 authority。0.6 对 `v0.5.0` 已发布的
  non-`Result` 签名不得静默改签名；其 legacy miss 行为必须在 compatibility matrix 中逐 symbol 记录，且 gateway
  production 不得使用该行为绕过 pricing policy。
- 仅对批准为 `deprecate_0_6_remove_0_7` 的 public symbol 添加 `#[deprecated(since = "0.6.0", ...)]`，
  保持签名、结果与 error contract。
- 用 `v0.5.0` baseline manifest 和下游式 compile fixture 证明每个已发布 import 在 0.6 head 仍可用；再覆盖
  authority/facade parity、provider alias/fallback、默认 `Reject` policy 的 unknown/incomplete pricing fail-closed，
  以及显式 `AllowUnpriced` policy 按配置 fallback cost 的 reservation/settlement parity，并同步 CHANGELOG 与迁移说明。

### Phase 3 — release 与 version workflow gate

- 记录包含 targeted deprecation 的已验证 0.6.x release artifact。
- version workflow 用 deterministic fixture 证明从 0.6.x breaking change 得到 0.7.0，而不是 1.0.0、patch
  或非 breaking label。
- public API owner 明确批准最终 removal 清单；`needs_decision` 项不得进入 removal。

### Phase 4 — 0.7 removal

- 只删除 Phase 2 已发布 deprecated 且 Phase 3 清单批准的 symbol/adapter/fallback。
- 删除后继续保留 `PricingService` authority、endpoint/spend semantics 与不在清单内的 provider-local catalog。
- 运行 public removal fixture、authority/facade regression、全量测试与 closure audit；任何 scope expansion 另开 spec。

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P1 | `pricing_service/authority.rs`, compatibility adapters, `spend/unpriced.rs`, spend/pricing routes | priced pricing/spend parity; default `Reject` fail-closed; `AllowUnpriced` reserve/settle parity |
| P2 | `v0.5.0` API manifest、approved `core::cost`/`core::pricing` exports and compatibility fixtures | tag/package-derived downstream compile fixture on 0.6 head |
| P3 | `provider_catalog_model_info`, Amazon Nova/xAI helpers, Azure/Bedrock/provider-owned catalogs and compatibility fallbacks | exhaustive call-graph/disposition guard plus provider alias/fallback tests |
| P4 | release/version workflow and approved removal list | deterministic 0.6.x → 0.7.0 fixture and public removal compile fixture |
| P5 | pricing usage/result conversion and provider regressions | cached/reasoning/multimodal/time-based focused tests |
| P6 | routes, persistence and network boundaries | exact-diff scope guard and existing route/spend tests |
| P7 | #519, #729, #965 references | roadmap reconciliation review |

## 数据流

调用输入仍是 provider、model、usage 与可选 modality/duration metadata。`PricingService` 从已加载数据解析价格，
并在 authority 内执行 inventory 已登记或批准迁入的 provider fallback 后返回 canonical result；0.6 compatibility
adapter 只把 canonical result/error 转成逐 symbol 批准的 legacy contract。authority miss 后 adapter 不得运行独立
catalog lookup。live gateway 在默认 `Reject` policy 下返回显式 not-found/incomplete-pricing error；只有显式
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
- Compatibility: Rust public import removal 是 breaking change；`core::pricing` 的 non-`Result` legacy API 也不得在
  0.6 静默改签名/错误合同；deprecation、迁移文档与 human approval 为硬门禁。
- Performance: adapter 不得每次重新解析 bundled pricing；继续复用已加载或 `LazyLock` authority。
- Maintenance: inventory guard 需要完整 path、authority call-graph root 与显式 disposition，避免同名 DTO 或
  provider-owned catalog 因目录位置被误判为无关。
- Overlap: `src/core/providers/**` 与 #837、router/registry 与 #965 均为默认禁止写入范围，除非后续 spec 明确缩小并重新 gate。

## 测试计划

- [ ] Inventory guard: `core::cost`/`core::pricing` public export、production consumer、
  `provider_catalog_model_info`/Amazon Nova/xAI helper 与全部 authority-reachable/legacy fallback 均有 disposition，
  新增 branch、decoy 与漏项负测试失败。
- [ ] Unit tests: authority-to-legacy DTO conversion、默认 `Reject` 的 unknown/incomplete pricing fail-closed、
  `AllowUnpriced` configured fallback、provider alias/fallback（Azure、Bedrock、Amazon Nova、xAI）。
- [ ] Integration tests: pricing route、budget reservation 与 spend settlement 对同一已定价 usage 保持 parity；
  `AllowUnpriced` 对同一未知 usage 的 reservation/settlement/usage-record cost 保持 parity。
- [ ] Compatibility: `v0.5.0` tag/package-derived `core::cost`/`core::pricing` public API manifest；0.6 downstream
  import compile/legacy behavior fixture；0.7 approved removal compile-fail/替代 import fixture。
- [ ] Version workflow: deterministic 0.6.x breaking fixture 产出 0.7.0。
- [ ] Repository: `cargo fmt --all -- --check`; `cargo check --all-targets --all-features --locked`; `cargo clippy --all-targets --all-features --locked -- -D warnings`; `cargo test --all-features --locked -- --test-threads=1`。

## 回滚方案

0.6 tranche 可整体 revert，恢复未标记 deprecated 的 compatibility exports；authority migration 必须与 adapter 变更一起回滚，
不能留下第二套 source。0.7 removal 通过独立 breaking PR 交付，可在未发布前 revert；发布后按 migration note 恢复 compatibility
adapter 需要新的 semver 决策。没有数据库迁移或数据回滚。
