# Tech Spec

## Linked Issue

GH-713

## Product Spec

Link to `product.md`.

## Current System

- `src/core/router/deployment.rs` stores `pub provider: Provider` in every
  router deployment.
- `src/core/providers/mod.rs` defines the closed `Provider` enum and dispatch
  macro arms that call each concrete provider implementation.
- `src/core/traits/provider/llm_provider/trait_definition.rs` documents
  `LLMProvider` as the core abstraction, including routing-oriented language.
- `src/core/traits/provider/handle.rs` stores an erased provider in
  `ProviderHandle`, but does not downcast or call it.
- `ProviderHandle` currently returns optimistic placeholder results for
  capability, health, cost, latency, and success-rate methods.

## Proposed Design

Choose the closed built-in provider contract for this PR.

1. Update module and type docs so they say the router deployment boundary is the
   `Provider` enum.
2. Update `LLMProvider` docs to describe it as the implementation interface for
   providers compiled and wired into the crate. Third-party implementations
   require crate integration before they can be routed.
3. Keep `ProviderHandle` for source compatibility, but describe it as a legacy
   metadata wrapper rather than a router dispatch path.
4. Change `ProviderHandle` methods to avoid fabricated success:
   - `supports_model` returns `false`.
   - `supports_tools` returns `false`.
   - `health_check` returns `HealthStatus::Unknown`.
   - `calculate_cost`, `get_average_latency`, and `get_success_rate` return
     `GatewayError::Internal` with a direct unsupported-contract message.
   - `chat_completion` continues to return an explicit error, with wording tied
     to the closed `Provider` enum route.
5. Add tests in `handle.rs` that construct a module-local test handle and assert
   the non-optimistic behavior.

## Alternatives

- Add `Provider::Custom(Arc<dyn DynProvider>)`: broader public API and routing
  change; defer until a custom-provider design is accepted.
- Replace `ProviderHandle` with a real object-safe adapter: useful long term,
  but out of scope for the narrow #713 contract alignment.
- Remove `ProviderHandle`: more breaking than necessary for this compatibility
  cleanup.

## Test Plan

- [x] Unit tests: `ProviderHandle` capability methods return false.
- [x] Unit tests: `ProviderHandle` health returns `Unknown`.
- [x] Unit tests: `ProviderHandle` cost/latency/success methods return
  explicit errors.
- [x] Build check: `cargo test provider_handle --lib`.
- [x] Build check: `cargo check --all-features --locked`.

## Rollback Plan

If the compatibility change breaks downstream users, keep the documentation
clarification and restore the old method bodies only behind explicit follow-up
review. No schema, feature, or persistent data migration is involved.
