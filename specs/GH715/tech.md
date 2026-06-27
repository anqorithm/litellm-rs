# Tech Spec

## Linked Issue

GH-715

## Product Spec

Link to `product.md`.

## Current System

- `src/core/providers/unified_provider.rs` defines `ProviderError` and legacy
  helpers for retryability, retry delay, and HTTP status.
- `src/server/routes/ai/execution.rs` retries selected deployments from
  `is_retryable_error` plus backoff delay.
- `src/core/router/execute_impl.rs` has the older router retry loops with the
  same context-free retry decisions.
- `src/server/routes/ai/openai_errors.rs` already maps `GatewayError` and
  `ProviderError` to OpenAI-compatible HTTP responses.
- Streaming route bodies emit SSE error events after stream creation; the
  selected-deployment retry loop only runs before a stream is returned.

## Proposed Design

1. Add `src/core/providers/failure.rs`:
   - `ProviderFailureKind`
   - `ProviderRetryHint`
   - `ProviderFailureFacts::from_error(&ProviderError)`
   These types expose provider name, failure kind, upstream status, and retry
   hints without owning policy decisions.
2. Add `src/core/router/retry_policy.rs`:
   - `RetryContext` with operation, stream stage, idempotency, attempt budget,
     and optional remaining deadline.
   - `RetryPolicy::decide(config, error, context)` returning a structured
     `RetryDecision`.
   - Provider `Retry-After` controls rate-limit retry delay when present.
   - `StreamRetryStage::AfterChunksEmitted` always stops automatic retry.
3. Update router and server execution:
   - Keep existing budget-fallback special case.
   - Replace direct retryability decisions with `RetryPolicy` for unary and
     pre-output streaming selected-deployment execution.
4. Keep HTTP mapping in `openai_errors.rs` and add adapter-boundary coverage.

## Compatibility

`ProviderError` variants and helper methods remain available. This PR changes
the runtime retry paths first, then leaves broad helper migration for a future
compatibility pass.

## Test Plan

- [x] `cargo test failure --lib`.
- [x] `cargo test retry_policy --lib`.
- [x] `cargo test budget_retry_fallbacks_skip_retry_delay --lib`.
- [x] `cargo test provider_timeout_http_mapping_lives_at_openai_adapter_boundary --lib`.
- [x] `cargo test execute_with_selected_deployment --lib`.
- [x] `cargo test core::router::tests::execution_tests --lib`.
- [x] `cargo check --all-features --locked`.

## Rollback Plan

Revert the new `failure` and `retry_policy` modules plus execution wiring. The
legacy `ProviderError` helper behavior remains intact, so rollback does not
require provider implementation changes.
