# Tech Spec

## Linked Issue

GH-838 / #838

## Product Spec

Link to `product.md`.

## Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| 启动入口 | `src/main.rs:103-114`、`src/server/builder.rs` | 仅 `tracing_subscriber::fmt()`；无 observability/langfuse/otel 初始化 | wire 的落点 |
| HTTP 装配 | `src/server/http.rs:198-223` | 中间件与路由注册全集；无 ip_access/guardrails/mcp/a2a/realtime | 可达性判定依据 |
| 配置根 | `src/config/models/gateway.rs:351-375` | `GatewayConfig` 字段全集；上述子系统无配置项 | invariant 2 的第一要件 |
| 未接线子系统 | `src/core/{guardrails,ip_access,mcp,a2a,realtime,webhooks,semantic_cache,analytics,virtual_keys,observability,integrations}` | 完整实现 + 测试，server/main 零引用 | 处置对象 |
| 合法库 API | `src/lib.rs`、`src/core/{completion,function_calling,traits,secret_managers}` | 通过 `pub mod core`、prelude 或 provider 内部 trait 使用暴露，不需要 server 路由 | guard 必须区分 library-only 与 gateway-facing |
| 公共模块导出 | `src/lib.rs`、`src/core/mod.rs` | `pub mod core` 暴露 `mcp`、`a2a`、`realtime` 等候选模块 | remove/gate 会影响下游 import，需 semver/CHANGELOG/deprecation |
| Batch 半接线 | `src/server/routes/ai/batches.rs:41-95` vs `src/core/batch/processor/core.rs:71,143,181` | 路由纯透传；`BatchProcessor` 从未构造 | 半接线样本 |
| virtual_keys | `src/core/virtual_keys/*`、`src/storage/database/migration/m20240301_000003_create_virtual_keys_table.rs`、`src/storage/database/seaorm_db/virtual_key_ops.rs` | 有迁移、manager 与 SeaORM CRUD；`src/core/mod.rs:49` 的 stub 注释已过期 | 应按「已实现但未挂 gateway」处置 |
| 自认证据 | `src/server/http.rs:660-669`（cache admin 501 "not wired"） | 代码自认部分管理面未接线 | 佐证 |

## 设计方案

**Phase 1 — 可达性证据表（附录）**

对每个 gateway-facing 子系统运行固定判定：
`rg "core::<name>|<MainType>" src/server src/main.rs src/bin src/config`，零命中即未接线。
表格记录：子系统、主类型、命中数、依赖的其他子系统、测试规模、最近 90 天 churn。另设
`library_only` 分类：若模块通过 `src/lib.rs`、prelude、provider trait 或 crate 内部 API 合法暴露
（例如 `completion`、`function_calling`、`traits`、`secret_managers`），不得仅因 server/main 零命中判为违规。

**Phase 2 — 处置矩阵（人工批复）**

预填建议（维护者可改）：

| 子系统 | 建议 | 理由 |
| --- | --- | --- |
| guardrails | wire（默认开，配置可关） | 安全语义，invariant 5 |
| ip_access | wire（中间件 + 配置） | 安全语义，代码量小 |
| observability + integrations | wire（启动初始化 + 配置） | 近期仍在投入（edec83d7），删除损失最大 |
| batch 持久化 | wire 完整化 or 删 processor（二选一） | 半接线态最误导 |
| mcp / a2a / realtime | experimental-gate | 实现大、无路由，产品化是独立决策 |
| webhooks / semantic_cache / analytics | remove or gate | stub 密度高，按维护者产品意向 |
| virtual_keys | wire or gate | 已有迁移与 storage-backed CRUD，但 gateway 管理/API 路径未接线；不能按 stub-only 删除 |

**Phase 3 — 执行**

- wire lane：每子系统一个 PR：`GatewayConfig` 字段 + `Default` + 校验 → 启动初始化（builder.rs）→
  中间件/路由挂载 → smoke 测试（U-26 checklist 全项）。
- gate lane：`Cargo.toml` 真 feature（`mcp = []` → gate `core/mcp` 的 `pub mod`）+ README/docs
  experimental 段；若 public import 改变，同步 semver、CHANGELOG、deprecation/迁移说明。
- remove lane：删除模块 + `core/mod.rs` 清理 + README/CLAUDE.md/`docs/` 同步；若 public import 改变，
  同步 semver、CHANGELOG、deprecation/迁移说明。

**Phase 4 — 守护检查**

脚本或测试：解析 `src/core/mod.rs` 的顶层 `pub mod` 清单，先分类为 `gateway_facing`、`library_only`、
`internal_support`、`feature_gated`。仅 `gateway_facing` 模块断言运行时可达：
「被启动装配实际构造并挂入请求路径/中间件/路由/后台任务 ∨ 在带 issue 的豁免清单 ∨ 被真 feature gate」。
单纯存在 `GatewayConfig` 字段、admin/status 展示、validation 文案或 `src/config`/`src/server` 文本引用不算可达性证据；
例如 `semantic_cache` 的配置与 admin flag 不能替代真实请求处理接线。CI 负测试必须证明新增 config-only
或 admin-only 的 gateway 子系统会失败，正测试必须证明 `completion`、`function_calling`、`traits`、
`secret_managers` 不会被误拦。

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P2 wire 三要件 | config + builder + http.rs | U-26 checklist 单测 + smoke 路由测试 |
| P3 remove 干净 | core/mod.rs + README/CLAUDE.md/`docs/` + CHANGELOG | `cargo check --all-features` + 全量测试 + public import/semver 记录 |
| P4 gate 真实 | Cargo.toml + cfg + docs.rs feature 列表 + CHANGELOG | `cargo check --no-default-features` 组合验证 + deprecation/迁移说明 |
| P5 安全默认 | guardrails/ip_access 配置 | 默认配置下中间件生效的集成测试 |
| P6 守护常驻 | CI 检查 | 人为添加未接线模块的负测试 + library-only 模块正测试 |

## 数据流

wire lane 引入新的启动初始化顺序：config load → storage → 各子系统 init → middleware 注册 → 路由。
observability 初始化必须在 server 启动前完成（tracing 全局注册的一次性约束），且配置感知 tracing/OTel/Langfuse
初始化必须发生在当前 fallback `tracing_subscriber::fmt().init()` 之前或替代它；不能先注册全局 fallback subscriber
再在 builder 中尝试安装配置化 subscriber。observability+integrations 还必须进入真实 LLM request 生命周期：
请求开始时触发 `IntegrationManager::on_llm_start`，成功/失败结束时触发 `on_llm_end`/`on_llm_error`，
不能只依赖既有 `/metrics` HTTP middleware。

## 备选方案

- 全部接线：mcp/a2a/realtime 产品化工作量数周起，且无用户需求证据，拒绝一刀切。
- 全部删除：observability/guardrails 是网关刚需能力且近期有投入，拒绝一刀切。
- 只改文档不动代码：消除宣传落差但维护成本继续发生，作为最低限度 fallback 记录。

## 风险

- Security: guardrails/ip_access 接线后默认开启可能改变现有部署行为——配置逃生门 + CHANGELOG。
- Compatibility: gate/remove 改变 `--all-features` 的模块集合或 `litellm_rs::core::<module>` public import，
  需同步 semver、CHANGELOG、docs.rs feature 列表（Cargo.toml 已有先例）与 deprecation/迁移说明。
- Performance: wire lane 新增中间件在热路径上，需按 #842 的分配纪律实现。
- Maintenance: 处置矩阵是一次性决策，守护检查防回归。

## 测试计划

- [ ] Unit tests: 各 wire 子系统的 U-26 三要件（config load 被调用、init 被调用、路由可达）。
- [ ] Integration tests: guardrails/ip_access 默认配置生效性。
- [ ] Observability integration tests: 对一条真实 chat/completion 请求注入 test integration，断言
      `on_llm_start` 与 `on_llm_end`/`on_llm_error` 被调用；`/metrics` 只能作为 HTTP middleware 辅助检查。
- [ ] Manual verification: `curl` 冒烟被 wire 的路由；Langfuse/OTel 或 test integration 记录请求生命周期事件。

## 回滚方案

wire lane 每子系统有独立配置开关，可运行时关闭；gate/remove lane 按 PR revert。
