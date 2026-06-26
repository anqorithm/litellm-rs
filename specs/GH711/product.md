# Product Spec

## Linked Issue

GH-711

## 用户问题

预算授权和实际记账分成两个步骤：请求先通过 `check_spend` 观察预算是否足够，稍后才用 `record_spend` 写入消费。并发请求在最后一段预算上会同时通过检查，最终集体超支。

预算金额还允许负数、NaN 和 infinite 进入边界，可能污染 `current_spend`、provider/model 预算统计和持久化快照。

## 目标

- 提供原子的预算 reservation API，让“授权”和“占用额度”在同一个临界区内完成。
- 支持 settle 实际金额，自动释放未使用的预留额度。
- 支持 drop/cancel 未结算 reservation 时归还额度。
- 在预算边界拒绝 negative、NaN、infinite 金额。
- 内部授权计算使用 fixed-point money；现有 `f64` 配置和展示输入必须显式转换和校验。
- 同一进程内多个并发请求争抢最后额度时，最多一个 reservation 成功。

## 非目标

- 不在本次实现跨进程强一致预算；Redis Lua 或数据库事务作为后续分布式方案。
- 不重写 pricing/token estimation 系统。
- 不改变现有公开配置字段的 `f64` 形状，除非需要新增兼容 API。
- 不把预算失败映射重构和 retry/error split 混入本 issue。

## 用户可见行为

当预算即将耗尽时，并发请求不会因为先后分离的检查和记账而共同越过预算上限。已授权但未完成的请求会先占住最大可消费额度；请求完成后按实际金额结算，未使用额度回到预算池。

非法金额输入会被明确拒绝或保持状态不变，不会把 NaN、infinite 或负数写入预算状态。

## 验收标准

- [x] `reserve_spend(scope, max_amount) -> BudgetReservation` 或等价 API 可用。
- [x] `BudgetReservation::settle(actual_amount)` 将最终消费记入预算并释放未使用额度。
- [x] 未 settle 的 reservation 在 drop/cancel 时释放全部预留额度。
- [x] negative、NaN、infinite 在 budget boundary 被拒绝。
- [x] 授权比较和加减使用 fixed-point money 或 Decimal。
- [x] 并发争抢最后预算的测试证明最多一个 reservation 成功。
- [x] provider/model/global budget 路径有一致的 reservation 语义或明确记录未接入边界。

## 边界情况

- 未配置预算时继续允许请求，但仍校验传入金额合法性。
- disabled budget 继续允许请求，不因 reservation 阻塞。
- `actual_amount > max_amount` 不能静默释放 reservation 或漏记；若 upstream 已完成，必须按实际 spend settle，并让预算状态反映 overage。
- pricing 失败或 usage 缺失时不应把 0 作为真实消费伪造记账，reservation 应释放。

## 发布说明

这是预算一致性修复。单进程内并发预算授权更严格；配置格式保持兼容。多进程部署仍需要后续分布式 reservation 后端才能获得全局强一致预算。
