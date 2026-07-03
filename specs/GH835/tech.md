# Tech Spec

## Linked Issue

GH-835 / #835

## Product Spec

Link to `product.md`.

## Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| OpenAI renderer | `src/server/routes/ai/openai_errors.rs:126-133` | `GatewayError::Config` -> 500/internal_error | Do not use for user bad model |
| Batch no provider | `src/server/routes/ai/batches.rs:368-372` | `missing_batch_provider_error()` returns Config | User-triggerable |
| Image no provider/model | `src/server/routes/ai/images.rs:430-448,696-701` | Missing image candidate returns Config | User-triggerable |
| Existing GatewayError | `src/utils/error/gateway_error/types.rs` | Has `BadRequest`, `NotFound`, `Provider(ModelNotFound)` | Prefer semantic variant |

## 设计方案

1. **Route-local semantic errors**：replace user-triggerable missing provider/model helpers with semantic 4xx variants:
   - batch provider absent: `GatewayError::BadRequest` or `ProviderError::NotSupported` wrapped for OpenAI shape;
   - image requested model unsupported/no candidate: `GatewayError::Provider(ProviderError::ModelNotFound { provider: "image_proxy", model })`
     or `GatewayError::BadRequest` with explicit OpenAI code mapping.
2. **No global Config remap**：do not change `GatewayError::Config` globally; startup/invalid URL/header config remains 500.
3. **Renderer coverage**：ensure chosen variant maps to OpenAI 4xx body. If current `openai_errors.rs` lacks desired code,
   add a route-specific helper or a narrow mapping test rather than broad Config remap.
4. **Test update**：change existing test that asserts 500 for missing provider/model to assert 4xx and non-internal code.

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P1 batch missing provider 4xx | batches.rs helper | Route/unit test status and code |
| P2 image no candidate 4xx | images.rs helper | Route/unit test status and code |
| P3 internal config remains 5xx | openai_errors.rs | Regression test Config still 500 |
| P4 body shape | OpenAI renderer | JSON shape assertion |

## 数据流

Request → route validates configured provider/candidate → user-triggerable absence becomes semantic 4xx →
OpenAI renderer returns client error body.

## 备选方案

- Globally map `GatewayError::Config` to 400: would hide real server misconfiguration，拒绝。
- Leave status 500 and change message only: still pollutes metrics and retries，拒绝。
- Convert every provider setup error to 4xx: invalid URL/header is operator error，拒绝。

## 风险

- Compatibility: Clients that retried these 500s will now see non-retryable 4xx.
- Maintenance: Need clear helper naming to prevent future user errors using `Config`.
- Observability: 5xx metrics become cleaner.

## 测试计划

- [ ] Unit/route tests for missing batch provider.
- [ ] Unit/route tests for image provider absent and model unsupported.
- [ ] Renderer regression test for real Config -> 500.

## 回滚方案

Single PR revert; status codes return to previous 500 behavior.
