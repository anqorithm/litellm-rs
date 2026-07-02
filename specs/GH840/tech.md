# Tech Spec

## Linked Issue

GH-840 / #840

## Product Spec

Link to `product.md`.

## Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| 现有半抽象 | `src/server/routes/ai/execution.rs:68` | `execute_with_selected_deployment` 只抽了 retry/选路 | 编排抽象的挂载点 |
| 样板站点 | `chat.rs:113-166, 416-482`、`completions.rs:152-231`、`embeddings.rs:113-198`、`images/generation.rs:39-90`、`audio/{speech.rs:81-139,transcriptions.rs:153-,translations.rs:141-}`、`gemini/provider.rs:315-`、`responses_stream.rs:78-140`、`moderations.rs:103`、`rerank.rs:81` | 各自 clone 4 Arc + reserve→call→settle 手写 | 迁移对象（~18 个 capture 点） |
| 结算实现 | `src/server/routes/ai/spend.rs`（`record_settled_spend` 等） | 结算细节（含 #831 修复后的语义） | 抽象的下游 |
| 预留实现 | `ensure_budget_available` / `reserve_*_budget` / `reserve_api_key_budget` | 预留三件套 | 抽象的上游 |
| State | `src/server/state.rs:46` | `key_manager` 按值 + 4 个独立字段 | 收敛为编排服务的机会 |

## 设计方案

1. **编排服务**：新增 `src/server/routes/ai/budgeted.rs`（或 `core/budget/orchestration.rs`）：

   ```rust
   pub struct BudgetedExecutor { /* 持有 4 组件的 Arc，一次构造进 AppState */ }

   pub enum SettlementMode { Metered, RecordOnly }  // rerank/moderations 用 RecordOnly

   impl BudgetedExecutor {
       pub async fn run<T, F>(
           &self,
           req: BudgetRequest<'_>,          // provider/model/key/预估 tokens/mode
           call: F,                          // 端点提供的 provider 调用
           usage_of: impl Fn(&T) -> Option<PricingUsage>,
       ) -> Result<T, GatewayError>
       where F: AsyncFnOnce() -> Result<T, GatewayError>;

       pub async fn run_stream(...) -> ...;  // 返回带 RAII 结算守卫的流包装
   }
   ```

2. **流式生命周期**：`run_stream` 返回 `SettledStream`：内部持有预留凭据，在
   「usage chunk 捕获 / 流终止 / drop」时按现状语义结算或退回（与 `chat.rs` 流式路径现有
   settle 时机逐行对照迁移）。

3. **AppState 收敛**：`AppState` 增加 `budgeted: BudgetedExecutor`（内部持 4 组件 Arc），
   端点侧不再直接触碰 4 组件；4 个旧字段保留到全部迁移完再评估收缩（U-01：不动公开 API 的前提下）。

4. **迁移策略**：每 PR 一个端点家族，先 chat 非 stream（最复杂的非流式）→ chat stream →
   其余端点机械迁移。每 PR 内新旧路径不并存（该端点整体切换），端点间允许新旧并存。

5. **类型强制（invariant 3）**：provider 执行入口函数改为 `BudgetedExecutor` 的方法参数
   回调，路由模块不再 export 裸执行函数。

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P1 分支行为等价 | budgeted.rs 全分支单测 | 预算不足/成功/失败退回/settle 失败四分支 + 迁移端点现有测试全绿 |
| P2 stream 生命周期 | SettledStream | 单测：usage 中段、无 usage 断开、错误终止三场景 |
| P3 类型强制 | 模块可见性 | `rg` 断言裸执行函数不再 pub |
| P4 样板消除 | 各端点 | `rg "budget_limits.clone\(\)" src/server/routes/ai` 零命中（budgeted.rs 除外） |

## 数据流

端点 handler → `BudgetedExecutor::run{,_stream}`（预留 → 回调调用 provider → usage 提取 → 结算/退回）
→ 响应。结算细节仍走 `spend.rs` 的既有函数（#831 语义在其内，抽象只负责编排顺序与生命周期）。

## 备选方案

- actix middleware 做预算编排：middleware 拿不到 usage（响应体已流式化），拒绝。
- 宏展开样板：可读性差且不解决「新端点忘加」问题，拒绝。
- 把编排塞进 `execute_with_selected_deployment`：该函数职责是 retry/选路，混入计费会复制
  god-function 问题，保持两层组合（budgeted 包裹 execute_with_selected_deployment）。

## 风险

- Security: 无新增面。
- Compatibility: 纯内部重构；风险在迁移偏差——靠 P1 分支对照与现有测试兜底。
- Performance: 减少 per-request Arc clone（顺带缓解 #842 的一部分）；无新增分配。
- Maintenance: 显著下降（计费语义单点）。

## 测试计划

- [ ] Unit tests: budgeted.rs 四分支 + SettledStream 三场景。
- [ ] Integration tests: chat/embeddings/images 带预算 key 端到端（迁移前后同断言）。
- [ ] Manual verification: 迁移每端点后 `cargo test --all-features` + 聚焦模块测试。

## 回滚方案

按端点 PR 逐个 revert；BudgetedExecutor 与旧样板可在端点粒度并存，无全局切换点。
