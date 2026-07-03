# Product Spec

## Linked Issue

GH-836 / #836

## 用户问题

Redis 分布式限流发生错误时，`RateLimiter` 只写 `warn!`，随后静默回退到进程内限流。
多节点部署下每个节点独立计数，实际全局限额扩大为节点数倍。操作员也没有 metric 能及时发现降级。

这是 user-visible correctness issue：限流承诺在 Redis 故障期间不再成立。

## 目标

- Redis 限流故障不再静默；必须 error 级日志 + metric。
- 操作者可显式选择 Redis 故障策略：fail-closed 或 fail-open-local。
- 默认策略保守，避免多节点下无声扩大限额。

## 非目标

- 不重写整个 rate limiter 算法。
- 不改变无 Redis / 单进程限流配置的行为。
- 不实现 Redis 自动恢复面板；只暴露状态与 metric。

## Behavior Invariants

1. 配置了 Redis 分布式限流且 Redis 操作失败时，gateway 必须记录 error 级日志并递增 degraded metric。
2. 默认模式为 fail-closed：Redis check/check_and_record 失败时返回 503 或限流型 429，不回退本地放行。
3. 只有显式配置 `fail_open_local` 时才允许本地 fallback；该 fallback 仍必须标记 degraded metric 和 response/log metadata。
4. `release` 失败也必须可观测，但不应导致已完成请求变成失败响应。
5. 没有 Redis backend 或 Redis pool 是 noop 时，现有进程内限流行为保持。

## 验收标准

- [ ] Redis `check` 失败默认 fail-closed，不走 local limiter。
- [ ] Redis `check_and_record` 失败默认 fail-closed，不创建 local reservation。
- [ ] `fail_open_local` 显式开启时才回退本地，且 metric/log 标记 degraded。
- [ ] `/metrics` 或 metrics collector 可观测 `rate_limiter_degraded_total{operation,mode}`。
- [ ] release 失败产生 error/metric，不影响响应完成。

## 边界情况

- Redis pool 初始化为 noop：不是故障，不打 degraded metric。
- Redis timeout 与 Redis command error 都算 degraded。
- 多节点部署文档/CHANGELOG 说明默认行为收紧与逃生门。

## 发布说明

Redis 分布式限流故障默认 fail-closed，并新增 degraded metric。需要旧的本地 fallback 行为时必须显式配置。
