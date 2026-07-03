# Tech Spec

## Linked Issue

GH-840 / #840

## Product Spec

Link to `product.md`.

## Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| 现有半抽象 | `src/server/routes/ai/execution.rs:68` | `execute_with_selected_deployment` 只抽了 retry/选路 | 编排抽象的挂载点 |
| 样板站点 | `chat.rs:113-166, 416-482`、`completions.rs:152-231`、`embeddings.rs:113-198`、`images/generation.rs:39-90`、`images.rs` image proxy、`audio/{speech.rs:81-139,transcriptions.rs:153-,translations.rs:141-}`、`gemini/provider.rs:315-`、`responses_stream.rs:78-140`、`moderations.rs:103`、`rerank.rs:81` | 各自直接访问 `state.budget_limits` / `state.pricing` / `state.key_manager` / `state.budget_manager`，并手写不同预算生命周期 | 迁移对象；最终 guard 不能只匹配 `.clone()` |
| 结算实现 | `src/server/routes/ai/spend.rs`（`record_settled_spend` 等） | 结算细节（含 #831 修复后的语义） | 抽象的下游 |
| 预留实现 | `ensure_budget_available` / `reserve_*_budget` / `reserve_api_key_budget` | 预留三件套 | 抽象的上游 |
| 流式选路租约 | `src/server/routes/ai/execution.rs` (`StreamingDeploymentLease`) | `finish_success(tokens)` / `finish_failure(error)` 才记录部署结果；`Drop` 只 release | `run_stream` 必须继续持有并显式完成租约 |
| State | `src/server/state.rs:46` | `key_manager` 按值 + 4 个独立字段 | 收敛为编排服务的机会 |

## 设计方案

1. **编排服务**：新增 `src/server/routes/ai/budgeted.rs`（或 `core/budget/orchestration.rs`）：

   ```rust
   pub struct BudgetedExecutor { /* 持有 4 组件的 Arc，一次构造进 AppState */ }

   pub enum SettlementMode {
       Metered,
       AvailabilityOnly,                 // moderations/rerank: 只保留 ensure_budget_available
       KeyReservationThenPostSuccessRecord, // image proxy: key 预留，成功后记 provider/model spend
   }

   pub struct SelectedDeploymentContext {
       pub provider: Provider,
       pub selected_model: String,
       pub deployment_id: String,
   }

   pub enum PreCallCharge {
       None,
       EstimatedUsage(PricingUsage),
       PricedUsage {
           pricing_provider: String,
           pricing_model: String,
           usage: PricingUsage,
           total_time_seconds: Option<f64>,
       },
       PrecomputedCost {
           usage: PricingUsage,
           cost: f64,
       },
   }

   impl BudgetedExecutor {
       pub async fn run<T, F>(
           &self,
           req: BudgetRequest<'_>,          // requested_model/capability/key/settlement_mode/pre_call_charge
           call: F,                          // cloneable per-attempt provider 调用
           usage_of: impl Fn(&T) -> Option<PricingUsage>,
       ) -> Result<T, GatewayError>
       where
           F: Fn(SelectedDeploymentContext) -> Fut + Clone,
           Fut: Future<Output = Result<(T, u64), ProviderError>>;

       pub async fn run_stream(...) -> ...;  // 返回显式异步 finalization 的流包装/流响应驱动
   }
   ```

   `call` 必须保持现有 retry/fallback 语义：每次尝试收到选中的 provider、model、deployment id，
   因此 rerank、native Gemini、provider-specific proxy 仍能用 selected deployment 上下文。

2. **流式生命周期**：`run_stream` 返回 `SettledStream` / response driver，但结算不能依赖 Rust `Drop`
   来执行 async 操作。包装器内部持有预留凭据、usage 状态、`saw_upstream_output`、可选
   `StreamingDeploymentLease`，并在每条可观测终止路径中显式 `.await` finalizer：
   - usage chunk：按实际 usage settle。
   - 正常结束且无 usage、但 `saw_upstream_output = true`：记录预留 spend（覆盖不发最终 usage 的 provider）。
   - 客户端断开：按当前端点路径记录或释放，不能新增未定义扣费。
   - 上游错误/转换错误/超时：若已经有 usage 或上游输出，沿用当前错误结算；若没有任何用户可见输出，则 cancel/drop reservation，不扣费。
   - 对当前会记录部署结果的路径继续调用 `StreamingDeploymentLease::finish_success(tokens)` 或
     `finish_failure(error)`；不能让 lease 只靠 `Drop` release 后丢失 success/failure。

3. **AppState 收敛**：`AppState` 增加 `budgeted: BudgetedExecutor`（内部持 4 组件 Arc），
   端点侧不再直接触碰 4 组件；4 个旧字段保留到全部迁移完再评估收缩（U-01：不动公开 API 的前提下）。

4. **迁移策略**：每 PR 一个端点家族，先 chat 非 stream（最复杂的非流式）→ chat stream →
   其余端点机械迁移。每 PR 内新旧路径不并存（该端点整体切换），端点间允许新旧并存。

5. **类型强制（invariant 3）**：provider 执行入口函数改为 `BudgetedExecutor` 的内部 driver 或方法参数回调。
   裸 `execute_with_selected_deployment` / `execute_stream_with_selected_deployment` 不能再以兄弟 route 可见的
   `pub(super)` API 暴露；若保留 helper，必须移动到 `budgeted` 内部或以模块结构限制为预算编排唯一调用方。

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P1 分支行为等价 | budgeted.rs 全分支单测 | 预算不足/成功/失败退回/settle 失败四分支 + 迁移端点现有测试全绿 |
| P2 stream 生命周期 | SettledStream / response driver | 单测：usage 中段、正常结束无 usage 但有输出、客户端断开、预输出错误退回、错误终止五场景；覆盖 lease `finish_success` / `finish_failure` |
| P3 类型强制 | 模块可见性 + import/call guard | `rg -n "(execution::execute_|execute_(with|stream)_selected_deployment\\()" src/server/routes/ai --glob '!budgeted.rs' --glob '!budgeted/**' --glob '!execution.rs' --glob '!*_tests.rs'` 除内部 driver 外零命中 |
| P4 样板消除 | 各端点 | `rg -n "state\\.(budget_limits|pricing|key_manager|budget_manager)\\b" src/server/routes/ai --glob '!budgeted.rs' --glob '!budgeted/**' --glob '!spend.rs' --glob '!spend/**' --glob '!*_tests.rs'` 对已迁移端点零命中；不只检查 `.clone()` |

## 数据流

端点 handler → 构造 `BudgetRequest`（含 capability、key 上下文、`SettlementMode`、`PreCallCharge`）
→ `BudgetedExecutor::run{,_stream}` 选择 deployment → cloneable per-attempt callback 调用 provider
→ usage 提取或使用预调用定价输入 → 显式 async settle/退回 → 响应。
结算细节仍走 `spend.rs` 的既有函数（#831 语义在其内，抽象只负责编排顺序与生命周期）。

### 端点模式

| 端点族 | 模式 | 必须保持的现有行为 |
| --- | --- | --- |
| chat / completions / embeddings / image generation / audio / gemini / responses_stream | `Metered` | provider/model 与 API-key 预留、成功结算、失败退回语义不变 |
| audio speech/transcription/translation | `Metered` + `PreCallCharge::PricedUsage` | 请求派生 `PricingUsage` 与 `total_time_seconds` 在 provider 调用前用于预留 |
| image generation | `Metered` + `PreCallCharge::PricedUsage` | 图片尺寸/质量/数量派生 usage 在 provider 调用前用于预留 |
| image edit / variation proxy | `KeyReservationThenPostSuccessRecord` + `PreCallCharge::PrecomputedCost` | provider/model 不做预调用 reservation；只预留 API key budget，成功后记录 provider/model spend |
| moderations / rerank | `AvailabilityOnly` | 只执行现有 `ensure_budget_available`；不新增 spend 或 key usage |

## 备选方案

- actix middleware 做预算编排：middleware 拿不到 usage（响应体已流式化），拒绝。
- 宏展开样板：可读性差且不解决「新端点忘加」问题，拒绝。
- 把编排塞进 `execute_with_selected_deployment`：该函数职责是 retry/选路，混入计费会复制
  god-function 问题，保持两层组合（budgeted 包裹 execute_with_selected_deployment）。
- 依赖 `Drop` 结算 stream：`Drop` 不能 `.await`，会漏掉 API-key usage / provider spend 记录，拒绝。

## 风险

- Security: 无新增面。
- Compatibility: 纯内部重构；风险在迁移偏差——靠 P1 分支对照与现有测试兜底。
- Performance: 减少 per-request Arc clone（顺带缓解 #842 的一部分）；无新增分配。
- Maintenance: 显著下降（计费语义单点）。

## 测试计划

- [ ] 单元测试：budgeted.rs 四分支 + 重试兼容 callback context + `PreCallCharge` 分支 + `AvailabilityOnly` 不记账。
- [ ] 流式测试：usage 中段、正常结束无 usage 但有输出、客户端断开、预输出错误退回、错误终止、lease success/failure。
- [ ] 集成测试：chat/embeddings/images 带预算 key 端到端（迁移前后同断言）。
- [ ] 守卫检查：direct AppState budget field access guard + sibling `execution::execute_*` import/call guard。
- [ ] 手工验证：迁移每端点后 `cargo test --all-features` + 聚焦模块测试。

## 回滚方案

按端点 PR 逐个 revert；BudgetedExecutor 与旧样板可在端点粒度并存，无全局切换点。
