# Product Spec

## Linked Issue

GH-724

## User Problem

The gateway still exposes overlapping internal and OpenAI-compatible chat/model
types. That makes it hard to know which type owns provider execution data and
which type only represents HTTP/OpenAI wire compatibility, increasing the risk
that fields are dropped during conversion.

## Goals

- Make `core::types::chat::ChatRequest` the documented internal chat request.
- Keep `core::models::openai::ChatCompletionRequest` as an OpenAI-compatible
  transport DTO with one explicit conversion boundary.
- Make `core::types::model::ModelInfo` the single internal model information
  authority.
- Add tests for OpenAI request fields that must survive transport-to-core
  conversion.

## Non-Goals

- Do not rewrite provider dispatch, provider construction, or registry behavior.
- Do not change SDK/HTTP provider parity beyond the type boundary.
- Do not remove OpenAI-compatible DTOs that are still needed for API
  compatibility.

## Behavior Invariants

1. Provider traits and provider implementations continue to receive
   `core::types::chat::ChatRequest`.
2. HTTP chat completion handlers continue to accept OpenAI-compatible JSON via
   `ChatCompletionRequest`.
3. Conversion from `ChatCompletionRequest` to `ChatRequest` preserves known
   routing, sampling, tool, metadata, service tier, stream option, and
   provider-specific extension fields.
4. The `ModelInfo` name under `core::models` no longer defines a second
   internal model-info shape.
5. Existing OpenAI-compatible request serialization behavior remains compatible.

## Acceptance Criteria

- [ ] Chat request ownership is documented at both internal and OpenAI DTO
      boundaries.
- [ ] `core::models::ModelInfo` resolves to the canonical
      `core::types::model::ModelInfo`, with any legacy shape renamed out of the
      authority path.
- [ ] Conversion tests cover currently risky request fields, including
      `store`, `metadata`, `service_tier`, `stream_options`, provider extension
      fields, tools, tool choice, functions, and function call.
- [ ] `cargo fmt --all -- --check` and `cargo test --lib` pass.

## Edge Cases

- OpenAI `seed` stays range-checked because the internal type is narrower.
- Route-level stream handling remains explicit: streaming and non-streaming
  handlers decide the internal `stream` flag.
- Unknown provider-specific request keys remain available through
  `ChatRequest::extra_params`.

## Rollout Notes

This is an internal type-boundary cleanup. It should not require migration for
HTTP clients because OpenAI-compatible DTOs remain in place.
