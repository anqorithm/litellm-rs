# Tech Spec

## Linked Issue

GH-1103 / #1103

## Product Spec

Link to `product.md`.

## Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| Canonical runtime authority | `src/core/pricing_service/authority.rs`, `src/core/pricing_service/service.rs`, `src/core/pricing_service/types.rs` | provider-aware loaded-data lookup and cost calculation power user-visible pricing paths | Must remain the single authority and replacement destination |
| Compatibility facade | `src/core/cost/mod.rs`, `src/core/cost/calculator.rs`, `src/core/cost/types.rs`, `src/core/cost/utils.rs` | exposes public DTOs/trait/functions and maps authority results into legacy shapes | Public lifecycle and adapter boundary under review |
| Legacy fallback catalogs | `src/core/cost/calculator/pricing.rs`, `src/core/cost/calculator/pricing/**`, `src/core/cost/providers/**` | authority miss may use provider-specific catalog logic | Each fallback needs evidence-backed disposition |
| Duplicate result shapes | `src/core/cost/types.rs:368`, `src/core/pricing_service/types.rs:55` | two `CostResult` types serve compatibility and authority layers | Same name does not prove safe deletion; conversion/consumer inventory is required |
| Public exports | `src/core/mod.rs`, `src/core/cost/mod.rs`, `src/core/pricing_service/mod.rs` | both modules are public | Downstream library imports create semver risk even when gateway runtime is unchanged |
| Live consumers | `src/server/routes/ai/spend.rs`, provider modules importing `core::cost`, pricing routes | GH726 routes live calculation through authority-backed helpers, while compatibility imports remain | Tests must distinguish authority calls from type/adaptor use |
| Predecessor packet | `specs/GH726/*` | deliberately retained legacy DTO/fallback compatibility after runtime convergence | This issue may narrow that deferral, not rewrite GH726 |

## 设计方案

按四个有序阶段执行；本 packet PR 只完成设计和任务拆分。

### Phase 1 — inventory 与守护

- 生成 tracked inventory，按完整 Rust path 记录 public re-export、production consumer、test-only consumer、
  DTO conversion 与 fallback owner。
- 从 `v0.5.0` tag 与已发布 package surface 生成 public API baseline manifest；当前树 inventory 必须显式说明
  baseline 中每个 symbol 是保留、重导出还是进入 deprecation，不能只扫描当前 head。
- public adapter disposition 只能是 `keep_adapter`、`deprecate_0_6_remove_0_7` 或 `needs_decision`；
  user-visible fallback disposition 只能是 `migrate_authority` 或 `needs_decision`。fallback 不得以
  `keep_adapter` 名义保留 lookup/calculation authority；空白或推断值 fail closed。
- source guard 必须在新增未登记 public export、production consumer 或 fallback 时失败；不得依赖字符串命中
  自动判定某项可删除。

### Phase 2 — 0.6 compatibility/deprecation

- `migrate_authority` 项把价格解析/匹配逻辑移到 `PricingService` authority 后，再由 `core::cost` adapter 映射
  legacy DTO；所有 user-visible fallback lookup/calculation 都必须在 authority 内运行，`core::cost` 只能转换
  DTO/error，不得复制或保留 catalog lookup 形成第二套 authority。
- 仅对批准为 `deprecate_0_6_remove_0_7` 的 public symbol 添加 `#[deprecated(since = "0.6.0", ...)]`，
  保持签名、结果与 error contract。
- 用 `v0.5.0` baseline manifest 和下游式 compile fixture 证明每个已发布 import 在 0.6 head 仍可用；再覆盖
  authority/facade parity、provider alias/fallback 和 unknown pricing fail-closed，并同步 CHANGELOG 与迁移说明。

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
| P1 | `pricing_service/authority.rs`, `core/cost` adapters, spend/pricing routes | pricing/spend parity tests and authority source guard |
| P2 | `v0.5.0` API manifest、approved `core::cost` exports and compatibility fixtures | tag/package-derived downstream compile fixture on 0.6 head |
| P3 | fallback inventory and `calculator/pricing/**` | exhaustive disposition guard plus provider alias/fallback tests |
| P4 | release/version workflow and approved removal list | deterministic 0.6.x → 0.7.0 fixture and public removal compile fixture |
| P5 | pricing usage/result conversion and provider regressions | cached/reasoning/multimodal/time-based focused tests |
| P6 | routes, persistence and network boundaries | exact-diff scope guard and existing route/spend tests |
| P7 | #519, #729, #965 references | roadmap reconciliation review |

## 数据流

调用输入仍是 provider、model、usage 与可选 modality/duration metadata。`PricingService` 从已加载数据解析价格，
并在 authority 内执行 inventory 批准迁入的 provider fallback 后返回 canonical result；0.6 `core::cost`
compatibility adapter 只把 canonical result/error 转成 legacy DTO/error。authority miss 后 adapter 不得运行独立
catalog lookup，必须返回显式 not-found/incomplete-pricing error，不能产生零成本默认值。
本工作不新增持久化、外部请求、后台任务或路由。

## 备选方案

- 立即删除整个 `core::cost`：拒绝；公开 import、provider consumer 与 0.6 兼容窗口未完成。
- 永久保留所有 duplicate DTO/fallback：拒绝；会让 #519 A-6 永久没有 lifecycle owner。
- 把所有 provider-local catalog 迁入一个 PR：拒绝；范围不可审查，并与 #837/provider-specific ownership 冲突。
- 重新实现 #726 authority convergence：拒绝；live authority 已完成，本 issue 只处理明确延期的兼容面。

## 风险

- Security: 不处理 secrets/auth；但错误 fallback 可能低估成本，必须保持 fail-closed。
- Compatibility: Rust public import removal 是 breaking change；0.6 deprecation、迁移文档与 human approval 为硬门禁。
- Performance: adapter 不得每次重新解析 bundled pricing；继续复用已加载或 `LazyLock` authority。
- Maintenance: inventory guard 需要完整 path 与显式 disposition，避免同名 DTO 误判。
- Overlap: `src/core/providers/**` 与 #837、router/registry 与 #965 均为默认禁止写入范围，除非后续 spec 明确缩小并重新 gate。

## 测试计划

- [ ] Inventory guard: public export、production consumer 与 fallback 全部有 disposition，decoy/漏项负测试失败。
- [ ] Unit tests: authority-to-legacy DTO conversion、unknown/incomplete pricing fail-closed、provider alias/fallback。
- [ ] Integration tests: pricing route、budget reservation 与 spend settlement 对同一 usage 保持 parity。
- [ ] Compatibility: `v0.5.0` tag/package-derived public API manifest；0.6 downstream import compile fixture；0.7 approved removal compile-fail/替代 import fixture。
- [ ] Version workflow: deterministic 0.6.x breaking fixture 产出 0.7.0。
- [ ] Repository: `cargo fmt --all -- --check`; `cargo check --all-targets --all-features --locked`; `cargo clippy --all-targets --all-features --locked -- -D warnings`; `cargo test --all-features --locked -- --test-threads=1`。

## 回滚方案

0.6 tranche 可整体 revert，恢复未标记 deprecated 的 compatibility exports；authority migration 必须与 adapter 变更一起回滚，
不能留下第二套 source。0.7 removal 通过独立 breaking PR 交付，可在未发布前 revert；发布后按 migration note 恢复 compatibility
adapter 需要新的 semver 决策。没有数据库迁移或数据回滚。
