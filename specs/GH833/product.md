# Product Spec

## Linked Issue

GH-833 / #833

## 用户问题

Provider、A2A、MCP 的 429 错误已经携带退避信息，但 gateway 返回给客户端时丢失
`Retry-After` / `X-RateLimit-*` 响应头。客户端 SDK 因此无法按上游建议退避，只能盲目重试。

当前证据：

- `ProviderError::RateLimit` 有 `retry_after`、`rpm_limit`、`tpm_limit` 字段。
- `GatewayError::Provider(ProviderError::RateLimit)` 在 response renderer 中只设置 status/code/message。
- `GatewayError::RateLimit` 分支才会写限流头。
- A2A/MCP conversion 把 `retry_after_ms` 写进 message，但 `retry_after: None`。

## 目标

- 所有 gateway 发出的 429 都保留可用的退避和限流头。
- Provider rate-limit、gateway rate-limit、A2A/MCP rate-limit 采用同一 header 写入逻辑。
- 不改变 #839 负责的错误 JSON 结构统一范围。

## 非目标

- 不新增完整限流策略或配额算法。
- 不重新设计 `GatewayError` / `ProviderError` 变体集合。
- 不保证所有 provider 都能提供 `Retry-After`；只保留已有事实。

## Behavior Invariants

1. `GatewayError::Provider(ProviderError::RateLimit { retry_after, rpm_limit, tpm_limit, .. })`
   返回 429 时写出对应 header。
2. `GatewayError::RateLimit { .. }` 的既有 header 行为不回退。
3. A2A/MCP 的 `retry_after_ms` 转换为 HTTP `Retry-After` 秒值，使用向上取整且最小 1 秒。
4. 没有 retry metadata 时不伪造 header，但 JSON message 可保持现状。
5. OpenAI-compatible AI 响应与 canonical 响应都能取得同一组 header facts，避免 #839 合并后再次漂移。

## 验收标准

- [ ] Provider 429 带 `Retry-After`、`X-RateLimit-Limit-Requests`、`X-RateLimit-Limit-Tokens`。
- [ ] Gateway 自身限流 429 既有 header 测试仍通过。
- [ ] A2A/MCP `retry_after_ms=1500` 转成 `Retry-After: 2`。
- [ ] 没有 retry metadata 的 429 不写空 header。

## 边界情况

- `retry_after_ms=0` 或小于 1000ms 时，HTTP header 使用 `1`，避免客户端立即重试。
- 只存在 rpm 或只存在 tpm 时，只写存在的 header。
- #839 统一错误映射后，header facts 仍应由单一 helper 提供。

## 发布说明

429 响应现在会保留退避 header，客户端可按 `Retry-After` 与 `X-RateLimit-*` 做正确重试。
