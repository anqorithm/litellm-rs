# Tech Spec

## Linked Issue

GH-831 / #831

## Product Spec

Link to `product.md`.

## Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| Spend settlement | `src/server/routes/ai/spend.rs:532-574` | pricing `Err` → `cost = None` → settle/record_spend 跳过、预留 drop 退回、`record_usage(.., 0.0)` | 主要缺口 |
| Spend pricing helper | `src/server/routes/ai/spend/pricing.rs:186` | 同模式 warn-then-None | 同步修复点 |
| Budget reservation | `src/core/budget/`（`reservation.settle` 调用方在 spend.rs） | 预留 drop 即退款 | invariant 3 的落点 |
| Pre-request check | `ensure_budget_available` / `reserve_*_budget`（chat/completions 等各端点） | 预留时不校验模型是否可定价 | fail-closed 的拦截点 |
| Pricing service | `src/core/pricing_service/service.rs` | `calculate_loaded_usage_cost_for_provider` 返回 `Err` | 需要区分「不可定价」错误类型的来源 |
| Config | `src/config/models/` | 无相关开关 | 新增策略配置 |

## 设计方案

1. **配置**：在 gateway pricing/budget 配置中新增
   `unpriced_model_policy: reject | allow_unpriced`（默认 `reject`），
   以及可选 `unpriced_fallback_cost_per_1k_tokens: Option<f64>`。
2. **fail-closed 拦截**：预算预留前增加一次「可定价性」检查（pricing service 暴露
   `can_price(provider, model) -> bool` 或等价 dry-run 接口）。`reject` 策略下返回
   `GatewayError` → OpenAI 错误形状 4xx（`model_not_priced` 语义），请求不发往 provider。
3. **结算语义修正**：`record_settled_spend`（spend.rs）中 pricing `Err` 分支改为：
   - `allow_unpriced` 策略：`cost = fallback_cost 或 0.0`，照常走 settle / `record_spend` /
     `settle_api_key_budget_reservation`，`record_usage` 附带 `unpriced: true` 标记；
   - 任何策略下不再出现「有 usage 但预留被退回」。
4. **可观测性**：新增 metric `gateway_unpriced_spend_total{provider,model,policy}`；
   error 日志保留现有文案并追加 policy 与处置结果。
5. 两处同模式路径（`spend.rs`、`spend/pricing.rs`）共用同一个结算辅助函数，避免再次漂移。

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P1 默认拒绝 | 预留前 can_price 检查 | 单测：unpriced 模型 + reject → 4xx，预算余额不变 |
| P2 放行但结算 | spend.rs Err 分支 | 单测：allow_unpriced → settle 被调用、退款不发生、usage 带标记 |
| P3 有 usage 必结算 | reservation 生命周期 | 单测：usage 存在时 reservation.settle 必被调用 |
| P4 可观测 | metrics | 单测/集成：metric 计数增加 |
| P5 已定价不变 | 全路径 | 现有 spend 测试回归 |

## 数据流

请求 → 预算预留（新增 can_price gate）→ provider 调用 → usage → `record_settled_spend`
→（定价成功：现状不变 | 定价失败：按 policy 结算并打标）→ per-key `record_usage`。

## 备选方案

- 仅升级日志为 error 并保留免费放行：不满足 invariant 1/3，预算控制仍然失效，拒绝。
- 在 pricing service 内部兜底返回 0 成本 `Ok`：把问题藏得更深，上层无法区分，拒绝。
- 全局硬编码 fail-closed 无配置：对私有部署（自建模型无定价）不友好，保留配置开关。

## 风险

- Security: 无新增面；收紧了绕过路径。
- Compatibility: 默认行为收紧，可能影响依赖未定价模型的现有部署——CHANGELOG + 配置逃生门。
- Performance: 预留前多一次 catalog 查找（内存哈希查找，可忽略）。
- Maintenance: 结算辅助函数统一两处路径，降低漂移风险。

## 测试计划

- [ ] Unit tests: spend.rs 结算分支（reject/allow、有无 fallback 价、有无 reservation）。
- [ ] Unit tests: can_price gate 与 4xx 错误形状。
- [ ] Integration tests: 带预算 key 的端到端 unpriced 请求（默认拒绝、开关放行）。
- [ ] Manual verification: `/metrics` 观察 `gateway_unpriced_spend_total`。

## 回滚方案

配置 `unpriced_model_policy: allow_unpriced` + 不设 fallback 价即可恢复接近旧行为（差异：spend
记录带标记且日志/metric 保留）。代码回滚为单 PR revert。
