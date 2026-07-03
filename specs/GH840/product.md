# Product Spec

## Linked Issue

GH-840 / #840

## 用户问题

在 `origin/main@c47596a4`，「预算预留 → provider 调用 → 结算」的编排样板在 AI 路由中复制了 10+ 处
（约 18 个 4-Arc capture 点：`budget_limits` / `pricing` / `key_manager` / `budget_manager`）：
chat（stream + 非 stream）、completions、embeddings、images、audio×3、gemini、responses_stream、
moderations、rerank。任何计费语义修改（例如 #831 的 fail-closed）都要同步修改 ~10 个文件，
极易漏改造成端点间计费口径不一致——这正是 #831 缺口能长期存在的结构性原因。

## 目标

- 一个统一的预算编排抽象；各端点只提供 provider 调用、预调用定价输入和响应 usage 提取逻辑。
- 计费语义（含 #831 落地后的 unpriced policy）只在一处实现，端点间不可能漂移。
- 迁移过程行为零变化（纯结构重构，U-07：不夹带行为修改）。

## 非目标

- 不改变任何计费语义（语义修正归 #831，先后关系见 handoff）。
- 不重构 `execute_with_selected_deployment` 的重试/选路逻辑。
- 不处理 SSE 流内部的 chunk 级结算细节（保持现有 settle 时机）。
- 不给 moderations / rerank 新增记账行为；不把 image proxy 改成 provider/model 预调用预留。

## Behavior Invariants

1. 迁移前后，每个端点在以下分支的可观测行为逐一相等：预算不足拒绝、预留成功+调用成功+结算、
   调用失败+预留退回、settle 失败的日志与 spend 记录。
2. 新抽象对 stream 与非 stream 两种生命周期都适用：非 stream 在响应前结算；stream 在终止时走显式异步 finalization，
   不依赖 `Drop` 执行 async 结算。stream 行为矩阵必须与现状一致：
   usage chunk 结算实际 usage；正常结束且无 usage 但已有上游输出时记录预留 spend；
   客户端断开按当前路径记录或释放；上游错误若发生在任何用户可见输出前则退回/丢弃预留而不是扣费。
3. 端点新增时不再可能绕过预算编排：能拿到 provider 执行入口的 API 就是编排抽象本身
   （类型上强制，而不是靠 review 记住加样板）；兄弟 route 不能直接 import/call `execution::execute_*`。
4. 4-Arc capture 样板从所有列出端点消除；`AppState` 侧只暴露一个编排入口。

## 验收标准

- [ ] 编排抽象（暂名 `BudgetedExecution` / `with_budget_settlement`）合入并有全分支单测。
- [ ] 列出的全部端点完成迁移，直接访问 `state.{budget_limits,pricing,key_manager,budget_manager}` 为零命中（编排/结算模块内部除外），不能只检查 `.clone()`。
- [ ] 兄弟 route 文件不能直接 import 或 call `execution::execute_with_selected_deployment` / `execute_stream_with_selected_deployment`；provider 执行必须从预算编排入口进入。
- [ ] 每个端点迁移 PR 附带该端点的行为对照证据（现有测试全绿 + 聚焦测试）。
- [ ] 端到端回归：带预算 key 的 chat/embeddings/images 三类请求行为不变。

## 边界情况

- moderations / rerank 等当前只做 `ensure_budget_available` 的端点：编排抽象必须提供显式 `AvailabilityOnly` / 无结算模式，
  只保留可用性检查，不新增 spend 记录或 key usage；若要改变语义必须另开 issue。
- gemini 原生路由（`gemini/provider.rs:315-`）与 OpenAI 兼容路由共用同一抽象。
- responses_stream 的 usage 出现在流中段：settle 时机保持现状（终止时），不提前。
- images/audio 的部分预留输入来自请求本身（`PricingUsage`、图片输出参数、音频 `total_time_seconds` 或预计算 cost），
  不能假设 usage 只能从 provider 响应中提取。
- image edit / variation proxy 当前是「可用性检查 + API key 预留 + 成功后 provider/model spend 记录」，
  迁移必须显式建模该模式，不能强制改成 provider/model 预调用预留。

## 发布说明

纯内部重构，无对外行为变化；CHANGELOG 以 refactor 记录。
