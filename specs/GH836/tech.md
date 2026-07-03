# Tech Spec

## Linked Issue

GH-836 / #836

## Product Spec

Link to `product.md`.

## Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| Redis check | `src/core/rate_limiter/limiter.rs:176-190` | Redis error -> warn -> local check | Silent fail-open |
| Redis check_and_record | `src/core/rate_limiter/limiter.rs:254-283` | Redis error -> warn -> local record | Multiplies quota |
| Redis release | `src/core/rate_limiter/limiter.rs:343-344` | release error handling is local to reservation | Must observe but not fail response |
| Config | `src/config/models/gateway.rs` / rate-limit config | No explicit degraded policy | Need operator choice |
| Metrics | `src/monitoring` / metrics middleware | No degraded counter | Need observability |

## 设计方案

1. **Config**：add explicit degraded policy under rate limit config:
   `redis_failure_mode: fail_closed | fail_open_local`, default `fail_closed`.
   Naming must make local fallback explicit; do not overload storage `allow_degraded`.
2. **check/check_and_record behavior**:
   - If Redis succeeds, unchanged.
   - If Redis fails and mode is `fail_closed`, return a denied/unavailable result without touching local limiter.
   - If Redis fails and mode is `fail_open_local`, run current local fallback.
3. **Observability**:
   - log at error level on Redis operation failure;
   - increment `rate_limiter_degraded_total{operation,mode}` for check/check_and_record/release;
   - optionally expose last degraded timestamp in health/debug state if local pattern exists.
4. **Release**：Redis release failure cannot change already-sent response, but must log error and metric.
5. **Tests**：use a fake Redis pool or trait seam. If current RedisPool is hard to fake, introduce a narrow internal trait for
   rate-limit Redis operations rather than hitting a real Redis service in unit tests.

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P1 no silent degrade | limiter.rs error branch | error log/metric test |
| P2 default fail-closed | config + limiter | Redis error returns denied/unavailable and no local increment |
| P3 explicit fallback | config + limiter | fail_open_local uses local limiter and marks degraded |
| P4 release observable | reservation/release path | release error increments metric but does not fail completed request |
| P5 no Redis unchanged | constructor/noop path | local limiter tests unchanged |

## 数据流

Request → RateLimitMiddleware → RateLimiter Redis operation →
success: distributed result; failure: degraded metric/log → policy decision fail_closed or explicit local fallback.

## 备选方案

- Keep warn + local fallback: violates no silent degradation，拒绝。
- Always fail-open but add metric: still violates configured global limit by default，拒绝。
- Require Redis for all rate limiting, no fallback option: too disruptive for single-node users，拒绝。

## 风险

- Compatibility: Default fail-closed is behavior tightening for Redis deployments; document escape hatch.
- Availability: Redis outage can reject traffic by default; this is intentional for global limit correctness.
- Maintenance: Avoid duplicating metrics code in two branches; use helper.

## 测试计划

- [ ] Config tests: default `fail_closed`, parse `fail_open_local`.
- [ ] Unit tests: Redis check failure in both modes.
- [ ] Unit tests: Redis check_and_record failure in both modes and reservation source.
- [ ] Unit tests: release failure metric/log path.
- [ ] Metrics test: degraded counter labels.

## 回滚方案

Set `redis_failure_mode=fail_open_local` to restore old runtime behavior with observability. Code revert restores silent fallback.
