# Product Spec

## Linked Issue

GH-838 / #838

## 用户问题

在 `origin/main@c47596a4`，一批完整子系统「声明但未接线」：代码、类型、测试俱全，但 `src/server`、
`src/main.rs`、`GatewayConfig` 中零引用，任何请求路径都不会执行它们：

- `core/guardrails`（内容安全护栏）——安全控制不生效；
- `core/ip_access`——未注册为中间件；
- `core/mcp`、`core/a2a`、`core/realtime`——无路由挂载，但 CLAUDE.md、README、`docs/README.md`、
  `docs/protocols/{mcp,a2a}.md` 以 "MCP Gateway"、"A2A Protocol" 对外宣传；
- `core/observability` + `core/integrations`（Langfuse/OTel）——`main.rs:103-114` 只初始化
  `tracing_subscriber::fmt()`，整套导出器无可达路径，且近期仍有 commit 在重构它（`edec83d7`）；
- `core/webhooks`、`core/semantic_cache`、`core/analytics`、`core/virtual_keys`；其中
  `virtual_keys` 已有迁移与 SeaORM CRUD，问题是 storage-backed 子系统已实现但未挂到 gateway 管理/API 路径；
- `/v1/batches` 纯透传，`core/batch::BatchProcessor` 持久化层从未被构造。

对用户的伤害：按文档宣传选型的用户拿到的是不存在的功能；对维护者的伤害：持续为不可达代码付出
重构、review、编译成本（U-26 declaration-execution gap）。

## 目标

- 每个子系统获得显式处置：接线（配置 + 启动初始化 + 路由/中间件挂载）或移出主干（删除 /
  experimental gate + 文档标注）。
- 文档（README、CLAUDE.md、`docs/README.md`、`docs/protocols/{mcp,a2a}.md`）能力描述与真实可达能力一致。
- 建立守护检查，防止新的「声明但未接线」子系统无声进入主干。

## 非目标

- 不在本 issue 内完成 MCP/A2A/realtime 的完整产品化（若维护者选择 wire，功能实现是后续独立 issue）。
- 不处理 provider 层的不可达目录（归 #837）。
- 不改变已接线子系统（router、budget、rate_limiter、cache 主路径等）的行为。

## Behavior Invariants

1. 处置矩阵批复前不删除任何子系统（U-05）。
2. 「wire」处置的子系统：存在配置项、启动初始化调用、以及至少一条端到端可达路径（路由或中间件），
   三者缺一即 CI 失败。
3. 「remove」处置的子系统：删除后 `cargo check --all-features` 与全量测试通过，文档同步更新。
4. 「experimental-gate」处置的子系统：模块被真实 feature gate（默认不编译），README 标注
   experimental 且不再出现在能力列表主表。
5. 安全语义类子系统（guardrails、ip_access）若保留，必须默认接线或在配置显式关闭——不允许
   「代码在但从不执行」的中间态。
6. remove/gate 若影响 `src/lib.rs` 暴露的 `pub mod core` 下公共模块，必须按 public API 变更处理：
   明确 semver 影响、CHANGELOG 条目、deprecation/迁移说明，不能只按「文档收缩」处理。
7. 守护检查常驻：`core/` 下 gateway-facing 子系统必须被 server/main 引用或在豁免清单（带 issue 引用）中；
   纯库 API 模块（如 `completion`、`function_calling`、`traits`、`secret_managers`）不得因没有 server 路由而被阻断。

## 验收标准

- [ ] 逐子系统处置矩阵（wire / remove / experimental-gate + 证据行）经维护者批复。
- [ ] 被保留子系统满足 invariant 2 并有 smoke 测试。
- [ ] 被移除/降级子系统的文档同步完成（README、CLAUDE.md、`docs/README.md`、
      `docs/protocols/{mcp,a2a}.md`），并完成 public API 影响记录。
- [ ] 守护检查合入 CI。

## 边界情况

- 子系统之间的依赖（如 observability 依赖 integrations）：处置必须按依赖拓扑成组决策。
- 半接线状态（batch：路由存在但绕过持久化层）：按「wire 完整化 or remove 持久化层」二选一，
  不允许维持绕过态。
- `.specrail/runtime`、`docs/` 中引用这些子系统的历史文档：不追溯修改，只改能力宣传文档。

## 发布说明

若选择 remove/gate，CHANGELOG 需标注能力宣传的收缩；若被处理模块仍通过 `src/lib.rs` → `pub mod core`
对外可导入，还需记录 semver/deprecation/迁移影响。即使 gateway 运行时路径原本不可达，也可能破坏下游
库用户的 `litellm_rs::core::<module>` import。
