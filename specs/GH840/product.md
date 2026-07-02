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

- 一个统一的 reserve→call→settle 编排抽象；各端点只提供 provider 调用与 usage 提取逻辑。
- 计费语义（含 #831 落地后的 unpriced policy）只在一处实现，端点间不可能漂移。
- 迁移过程行为零变化（纯结构重构，U-07：不夹带行为修改）。

## 非目标

- 不改变任何计费语义（语义修正归 #831，先后关系见 handoff）。
- 不重构 `execute_with_selected_deployment` 的重试/选路逻辑。
- 不处理 SSE 流内部的 chunk 级结算细节（保持现有 settle 时机）。

## Behavior Invariants

1. 迁移前后，每个端点在以下分支的可观测行为逐一相等：预算不足拒绝、预留成功+调用成功+结算、
   调用失败+预留退回、settle 失败的日志与 spend 记录。
2. 新抽象对 stream 与非 stream 两种生命周期都适用：非 stream 在响应前结算；stream 在终止
   （最后 usage chunk / 客户端断开 / 错误）时结算，语义与现状一致。
3. 端点新增时不再可能绕过预算编排：能拿到 provider 执行入口的 API 就是编排抽象本身
   （类型上强制，而不是靠 review 记住加样板）。
4. 4-Arc capture 样板从所有列出端点消除；`AppState` 侧只暴露一个编排入口。

## 验收标准

- [ ] 编排抽象（暂名 `BudgetedExecution` / `with_budget_settlement`）合入并有全分支单测。
- [ ] 列出的全部端点完成迁移，`state.{budget_limits,pricing,key_manager,budget_manager}.clone()` 样板搜索为零命中（编排模块内部除外）。
- [ ] 每个端点迁移 PR 附带该端点的行为对照证据（现有测试全绿 + 聚焦测试）。
- [ ] 端到端回归：带预算 key 的 chat/embeddings/images 三类请求行为不变。

## 边界情况

- moderations / rerank 等无逐 token 计费的端点：编排抽象允许「无预留、仅记账」的退化模式，显式声明而非隐式跳过。
- gemini 原生路由（`gemini/provider.rs:315-`）与 OpenAI 兼容路由共用同一抽象。
- responses_stream 的 usage 出现在流中段：settle 时机保持现状（终止时），不提前。

## 发布说明

纯内部重构，无对外行为变化；CHANGELOG 以 refactor 记录。
