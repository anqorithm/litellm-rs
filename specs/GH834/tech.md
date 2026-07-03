# Tech Spec

## Linked Issue

GH-834 / #834

## Product Spec

Link to `product.md`.

## Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| Image generation route | `src/server/routes/ai/images.rs:60-72` | authz only inside `if let Some(model)` | Missing-model bypass |
| Image proxy paths | `src/server/routes/ai/images.rs:130-137` | multipart model required and checked | Should remain unchanged |
| Authz helper | `src/server/routes/ai/context.rs:250` | Existing API key allowed_models/token limit enforcement | Reuse, do not fork policy |
| Provider defaults | provider image implementations | Many providers choose defaults internally | Must resolve or fail before provider |

## 设计方案

1. **effective model resolver**：在 image generation route 增加 helper，例如
   `resolve_image_generation_authz_model(request, providers) -> Result<Option<String>, GatewayError>`。
   - If request has `model`, return it.
   - If request has no `model` and key has no model restriction, return `None` and preserve old default behavior.
   - If request has no `model` and key has model restriction, resolve the single configured image generation default/candidate model
     if deterministic; otherwise return 4xx requiring explicit `model`.
2. **authz before provider**：call `enforce_api_key_model_and_token_limits` with resolved effective model before
   `handle_ai_request` invokes provider code.
3. **fail-closed ambiguity**：multiple possible provider defaults, no configured default, or provider-only hidden defaults
   are not safe for restricted keys. Return OpenAI-compatible 4xx (`invalid_request` or `model_not_allowed`) without
   touching provider.
4. **test doubles**：use route tests with a request extension/API key context and a sentinel provider/router to prove provider
   is not invoked on denied missing-model requests.

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P1 no bypass | images.rs route | Missing model + restricted key denied before provider |
| P2 deterministic default allowed | resolver | Default in allowed_models passes authz |
| P3 ambiguity fail-closed | resolver | Multiple/unknown defaults return 4xx |
| P4 unrestricted unchanged | route | No allowed_models keeps old no-model behavior |

## 数据流

Request JSON → resolve explicit/effective authz model → enforce API key model policy if needed →
existing `handle_ai_request` → provider/routing.

## 备选方案

- Require `model` for every image generation request: safe but breaks unrestricted clients unnecessarily，拒绝。
- Check after provider picks default: too late; provider call already escaped authz，拒绝。
- Duplicate allowed_models matching in images.rs: risks drift from central helper，拒绝。

## 风险

- Security: Fixes privilege bypass; fail-closed for ambiguous restricted-key cases.
- Compatibility: Restricted keys omitting model may now get 4xx; unrestricted keys unchanged.
- Maintenance: Resolver must stay tied to routing/provider default semantics.

## 测试计划

- [ ] Unit tests: effective model resolver explicit/default/ambiguous cases.
- [ ] Route tests: missing model with restricted key denied before provider.
- [ ] Regression tests: explicit model allow/deny unchanged.

## 回滚方案

Single PR revert; revert reopens allowed_models bypass for image generation missing-model requests.
