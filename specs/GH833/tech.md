# Tech Spec

## Linked Issue

GH-833 / #833

## Product Spec

Link to `product.md`.

## Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| Provider facts | `src/core/providers/unified_provider_error.rs:11-19` | `RateLimit` carries retry/rpm/tpm | Header source exists |
| Response renderer | `src/utils/error/gateway_error/response.rs:31-36,207-226` | Provider 429 does not insert headers; gateway 429 does | Missing branch |
| A2A conversion | `src/utils/error/gateway_error/conversions.rs:84-101` | `retry_after_ms` only in message, `retry_after: None` | Metadata lost |
| MCP conversion | `src/utils/error/gateway_error/conversions.rs:210-227` | Same as A2A | Metadata lost |
| Future mapping | `specs/GH839/` | Plans canonical HTTP facts | This fix must be compatible |

## 设计方案

1. **Header facts helper**：新增或复用一个 small helper，例如
   `rate_limit_headers(error: &GatewayError) -> RateLimitHeaderFacts`，覆盖：
   - `GatewayError::RateLimit`;
   - `GatewayError::Provider(ProviderError::RateLimit)`.
2. **Response renderer 接入**：`GatewayError::error_response` 在构造 429 response 时调用 helper，
   不再只匹配 `GatewayError::RateLimit`。
3. **A2A/MCP ms 转秒**：conversion 中把 `retry_after_ms: Option<u64>` 转为
   `retry_after: Option<u64>` 秒值：`ceil(ms / 1000)`，且 `Some(0)` 变为 `Some(1)`。
4. **#839 兼容**：如果 #839 的 `http_facts` 先落地，本 issue 的 helper 应并入 facts；如果本 issue 先落地，
   #839 迁移时必须保留这些 header tests。

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P1 Provider headers | response.rs helper | Unit test ProviderError::RateLimit emits headers |
| P2 Gateway headers | existing branch | Regression test existing GatewayError::RateLimit |
| P3 A2A/MCP ms conversion | conversions.rs | Unit tests 1500ms -> 2s, 1ms -> 1s |
| P4 no fake headers | helper | Unit test metadata None writes no headers |

## 数据流

Provider/A2A/MCP error → `GatewayError` with structured retry facts → renderer/header facts helper →
HTTP 429 response with JSON body and rate-limit headers.

## 备选方案

- Parse retry seconds back out of message strings: fragile and locale-dependent，拒绝。
- Wait for #839 only: leaves current bug open and risks losing facts before the refactor，拒绝。
- Always set fixed `Retry-After`: invents metadata，拒绝。

## 风险

- Compatibility: Adds headers only; body/status remain unchanged.
- Security: No sensitive data; rpm/tpm values are already intended rate-limit metadata.
- Maintenance: Keep helper close to #839 facts to avoid third mapping table.

## 测试计划

- [ ] Unit tests: provider rate limit headers.
- [ ] Unit tests: gateway rate limit headers remain.
- [ ] Unit tests: A2A/MCP retry_after_ms conversion.
- [ ] Integration/focused route test if existing test harness can trigger 429.

## 回滚方案

Single PR revert. Reverting loses retry headers but restores previous body/status behavior.
