# Tech Spec

## Linked Issue

GH-711

## Product Spec

Link to `product.md`.

## 当前系统

- `src/core/budget/tracker.rs` 通过 `DashMap<String, Budget>` 存储 scope budget，`check_spend` 只读，`record_spend` 之后才写。
- `src/core/budget/provider_limits.rs` 为 provider/model 维护独立 budget manager，也有 `can_*_spend` 与 `record_*_spend` 分离的问题。
- `src/core/budget/types.rs` 的 `Budget`、`ProviderBudget`、`ModelBudget` 公开字段使用 `f64`，现有 API 直接加到 `current_spend`。
- `src/server/routes/ai/chat.rs` 和 `src/server/routes/ai/spend.rs` 的真实请求路径只在 provider call 前检查 exhausted，完成后才按 usage/pricing 记账。

## 设计方案

1. 增加 fixed-point money 边界类型，例如 `BudgetAmount`，负责从 legacy `f64` 显式转换并拒绝 negative、NaN、infinite。
2. 保留公开配置/展示结构的 `f64` 字段，所有授权比较、预留、释放和 settle 使用 `BudgetAmount` 做加减和比较。
3. 在 `BudgetTracker` 上增加 `reserve_spend(scope, max_amount)`，在 `DashMap::get_mut` 持有期间完成：
   - 校验金额。
   - 检查剩余额度。
   - 将 `max_amount` 加入 `current_spend` 作为预留。
   - 返回持有 tracker clone、scope、reserved amount、settled flag 的 `BudgetReservation`。
4. `BudgetReservation` 支持：
   - `settle(actual_amount)`：将预留额度替换成实际消费；若 provider 实际消费超过预留，也按实际金额记账，避免释放成漏记。
   - `cancel()`：释放全部预留额度。
   - `Drop`：未 settle/cancel 时释放全部预留额度。
5. 为 provider/model manager 提供同等 reservation API，并在 `UnifiedBudgetLimits` 上组合 provider + model reservation；如果 model reservation 失败，provider reservation 必须自动释放。
6. 在 `BudgetAwareRouter` 的 estimated-cost 入口上提供 reservation API。chat completion 的 non-streaming、chat streaming、Responses API streaming 路径都在 upstream 前预留，完成后的真实 usage/cost 通过 reservation settle。
7. 保留旧 `record_spend` 兼容 API，但让它在非法金额时拒绝变更，避免污染状态。
8. 对未设置 `max_tokens` / `max_completion_tokens` 的 chat request，预算预留使用 token counter 的模型上下文剩余额度作为保守输出上限，而不是使用 cost calculator 的 100-token 默认估算。

## 数据流

```text
request max amount
  -> reserve_spend
  -> current_spend += reserved_max
  -> upstream work
  -> settle(actual)
  -> current_spend = current_spend - reserved_max + actual
```

无预算配置：

```text
validate amount -> no tracked mutation -> reservation drop no-op
```

provider/model 组合：

```text
reserve provider -> reserve model -> UnifiedBudgetReservation
model reserve failure -> provider reservation drop/cancel
```

## 备选方案

- 使用 Decimal crate：更直接表达 money，但会扩大依赖面；本 issue 可用固定精度整数降低引入风险。
- 只修 middleware：不能覆盖真实 chat/provider spend 路径，不能满足 issue。
- 只在 `record_spend` 加锁：仍然无法阻止多个请求同时通过 precheck。

## 风险

- Security: 非法金额拒绝必须在所有 budget boundary 一致执行，避免 NaN/infinite 污染状态。
- Compatibility: 保留 `f64` public fields，新增 API 不应破坏现有配置序列化。
- Performance: reservation 使用现有 `DashMap` mutable entry，范围短，只做金额校验和状态更新。
- Maintenance: provider/model/global 三条 budget 路径容易漂移；测试需要覆盖三者。

## 测试计划

- [x] Unit tests: `BudgetAmount` 拒绝 negative、NaN、infinite；fixed-point 加减比较正确。
- [x] Unit tests: `BudgetTracker::reserve_spend` settle/cancel/drop 行为。
- [x] Unit tests: 并发 N 个线程争抢最后额度，最多一个 reservation 成功。
- [x] Unit tests: provider/model/UnifiedBudgetLimits reservation 组合释放行为。
- [x] Integration tests: `BudgetAwareRouter` estimated-cost reservation；spend route priced/unpriced 回归。
- [x] Regression tests: actual spend 大于 reservation 时按实际金额记账；未设置 max tokens 时使用保守输出上限；Responses stream helper tests 拆分后继续执行。
- [x] Manual verification: `cargo test budget --lib`、`cargo test spend --lib`、`cargo test responses_stream --lib`、`cargo check --all-features --locked`、`cargo test --lib`。

## 回滚方案

若 reservation 接入引发兼容问题，可以保留 fixed-point 校验但临时回退真实请求路径到旧 `record_spend`，同时保留新增 tests 标记待修。由于不改持久化 schema，回滚不需要迁移数据。
