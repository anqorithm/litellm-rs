# Product Spec

## Linked Issue

GH-715

## User Problem

Provider failures currently carry retry and HTTP response policy as convenience
methods on `ProviderError`. That makes router behavior depend on an error
variant without enough request context: retry budget, stream stage, idempotency,
deadline, and provider retry hints.

## Goals

- Introduce fact-only provider failure data that can be derived from
  `ProviderError` without owning retry or HTTP policy.
- Add a router retry policy layer that receives request context before deciding
  whether and when to retry.
- Ensure streaming failures after client-visible output are not automatically
  retried.
- Ensure pre-output streaming failures can still retry when policy and budget
  permit.
- Keep HTTP status mapping at the OpenAI-compatible gateway adapter boundary.
- Preserve existing provider implementations and `ProviderError` compatibility
  helpers.

## Non-Goals

- Do not remove or rename `ProviderError` variants in this PR.
- Do not migrate every provider-specific error test away from legacy helper
  methods.
- Do not change the broad `LLMProvider` trait shape; that remains #519 scope.
- Do not rewrite retry/cooldown selection or budget fallback routing.

## User-Visible Behavior

Successful requests are unchanged. Retry behavior gains explicit policy inputs:
`Retry-After` can control retry delay, and streaming errors after emitted chunks
are classified as non-retryable by policy.

## Acceptance Criteria

- [x] SpecRail docs record the #715 migration slice.
- [x] Provider failure facts can be derived from `ProviderError` without policy
  decisions.
- [x] Router selected-deployment helpers call `RetryPolicy` instead of deciding
  only from the error variant.
- [x] Tests cover post-output streaming no-retry and pre-output streaming retry.
- [x] Tests cover provider `Retry-After` in delay calculation.
- [x] HTTP mapping tests live in `server/routes/ai/openai_errors.rs`.

## Follow-Up

A later major-version cleanup can deprecate or remove `ProviderError`
compatibility helpers once SDK/provider tests and external callers use the new
fact and adapter surfaces.
