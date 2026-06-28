# Tech Spec

## Linked Issue

GH-724

## Product Spec

See `product.md`.

## Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| Internal chat request | `src/core/types/chat.rs` | Defines `ChatRequest` used by provider traits and providers. | This is the canonical provider execution request. |
| OpenAI transport request | `src/core/models/openai/requests.rs` | Defines `ChatCompletionRequest` for OpenAI-compatible JSON. | This should stay a transport DTO, not an internal authority. |
| HTTP conversion boundary | `src/server/routes/ai/chat.rs` | `build_core_chat_request` converts the DTO into `ChatRequest`. | This is where field preservation must be explicit and tested. |
| Model information | `src/core/types/model.rs`, `src/core/models/mod.rs` | Both define a `ModelInfo` shape; providers use the one under `core::types`. | The duplicate authority named `ModelInfo` must be removed from `core::models`. |

## Proposed Design

Document the ownership boundary in the two request modules and in the HTTP
conversion function. Keep `ChatCompletionRequest` as the HTTP/OpenAI DTO and
`ChatRequest` as the provider-facing internal type.

Rename the old `core::models::ModelInfo` struct to `LegacyModelSummary` because
it is not used by provider traits and is not the canonical model-info authority.
Then expose `pub type ModelInfo = crate::core::types::model::ModelInfo` from
`core::models` so the public name resolves to the canonical shape.

Expand the existing `build_core_chat_request` tests to cover the fields at risk
of conversion loss. Keep seed overflow validation as the explicit lossy/narrow
case.

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P1 | Provider-facing `ChatRequest` docs and route conversion | `cargo test --lib test_build_core_chat_request_minimal` |
| P2 | OpenAI DTO docs and route conversion | `cargo test --lib test_build_core_chat_request_preserves_transport_fields` |
| P3 | `core::models::ModelInfo` alias to canonical type | `cargo test --lib test_core_models_model_info_uses_canonical_shape` |
| P4 | Legacy duplicate is renamed out of authority path | `cargo test --lib test_legacy_model_summary_structure` |

## Data Flow

HTTP JSON enters as `ChatCompletionRequest`, route validation runs on the DTO,
and `build_core_chat_request` produces the internal `ChatRequest` used by
routing/provider execution. Provider-specific extension fields flow into
`ChatRequest::extra_params`; explicit fields stay on explicit internal fields.

No persistence, network behavior, or external API behavior changes in this
slice.

## Alternatives Considered

- Removing OpenAI request DTOs entirely: rejected because the HTTP API still
  needs OpenAI-compatible serialization and deserialization.
- Moving conversion into `core::models::openai`: rejected because it would make
  OpenAI DTO modules depend on server-layer error types.
- Deleting the legacy model summary struct outright: rejected to keep an
  explicit compatibility type for any internal callers that still need that
  summary shape later.

## Risks

- Security: No new secret, auth, SQL, or command execution surface.
- Compatibility: The `core::models::ModelInfo` public name now points at the
  canonical model-info shape; this is intentional for GH-724 but may affect
  consumers of the duplicate legacy fields.
- Performance: Conversion remains linear over messages/tools/functions.
- Maintenance: The legacy summary type is clearly named so it cannot be
  mistaken for the internal model-info authority.

## Test Plan

- [ ] Unit tests: focused model and chat conversion tests.
- [ ] Integration tests: covered by `cargo test --lib` for this internal slice.
- [ ] Manual verification: SpecRail packet validation and PR gate evidence.

## Rollback Plan

Revert the PR. The change is contained to type definitions, documentation, tests,
and the explicit transport-to-core conversion boundary.
