# Tech Spec

## Linked Issue

GH-726 / #726

## Product Spec

Link to `product.md`.

## Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| Runtime pricing service | `src/core/pricing_service/service.rs`, `types.rs`, `loader.rs` | Owns loaded LiteLLM pricing data and powers `/v1/pricing/*`, but only supports model-key lookup for calculation. | This should become the authority for calculation and provider/model matching. |
| Shared pricing DB | `src/core/pricing.rs` | Exposes `PricingDatabase`, model normalization, provider normalization, bundled defaults, and standalone `calculate_cost`. | Existing parsing and normalization should remain shared; duplicate runtime calculation should move behind PricingService-compatible logic. |
| Legacy cost facade | `src/core/cost/calculator.rs`, `types.rs`, `utils.rs` | Owns `generic_cost_per_token`, `estimate_cost`, `get_model_pricing`, `CostBreakdown`, and provider-specific fallback logic. | Keep public compatibility, but convert calculation to a PricingService-backed adapter. |
| AI spend path | `src/server/routes/ai/spend.rs` and spend tests | Budget reservation and settlement import `core::cost::calculator` directly. | User-visible spend must use the same pricing authority as the pricing route. |
| Pricing route | `src/server/routes/pricing.rs` | `/v1/pricing/calculate` calls `AppState.pricing.calculate_completion_cost`. | This is the existing route that must stay behaviorally aligned with spend. |
| Provider-specific tests | `src/core/cost/calculator/tests.rs`, `pricing_regression_tests.rs`, AI spend tests | Tests already cover provider aliases and strict missing pricing. | Reuse and extend these to prove compatibility after convergence. |

## Proposed Design

Keep `PricingService` as the canonical user-facing authority. Add synchronous, already-loaded-data
helpers on `PricingService` so both HTTP pricing routes and legacy cost adapters can resolve and calculate
without requiring a server `AppState`:

- Add provider-aware lookup and calculation helpers to `PricingService`.
- Add a default authority constructor for compatibility callers that need bundled pricing outside server state.
- Convert `core::cost::calculator::{get_model_pricing, generic_cost_per_token, estimate_cost}` to call
  the PricingService-backed helpers and then map results into legacy DTOs.
- Keep `UsageTokens`, `CostBreakdown`, and `CostEstimate` as compatibility DTOs, not an independent
  pricing authority.
- Update `src/server/routes/ai/spend.rs` so live reservation and settlement helpers receive
  `AppState.pricing` and calculate from the runtime `PricingService`; keep test-only wrapper names for
  existing unit tests.

Provider-specific hardcoded fallbacks may remain only inside the PricingService authority where the
shared catalog cannot resolve a model and the existing behavior already used a provider catalog, such as
Azure and Bedrock compatibility pricing. Unknown or incomplete shared catalog rows must not fall through
to $0 billing.

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P1 | `pricing_service/service.rs`, `cost/calculator.rs`, `server/routes/pricing.rs`, `server/routes/ai/spend.rs` | `cargo test pricing --lib --locked`, `cargo test spend --lib --locked` |
| P2 | `cost/calculator.rs` | `cargo test estimate_cost --lib --locked`, existing spend reservation tests |
| P3 | `pricing_service/service.rs`, `cost/calculator.rs`, spend tests | missing-pricing focused tests and `cargo test spend --lib --locked` |
| P4 | `cost/calculator.rs`, `cost/types.rs` | legacy cost calculator tests continue to pass |
| P5 | provider alias helpers and existing provider-specific fixtures | `cargo test runtime_pricing --lib --locked` |

## Data Flow

Pricing data is loaded from LiteLLM JSON through the existing parser into `PricingService`.
Cost inputs are provider, model, usage tokens, optional prompt/completion text, and optional duration.
The authority resolves the best matching pricing row, validates required fields, calculates input/output
and extra token costs, then returns either `CostResult` or a legacy DTO adapter. No new persistence,
external network calls, or background services are added.

## Alternatives Considered

- Remove `core::cost` entirely. Deferred because many provider modules and tests import its DTOs and
  facade functions.
- Move spend signatures to accept `Arc<PricingService>`. Deferred for this slice because the existing
  call graph can be converged by making the facade authoritative first.
- Rewrite pricing provider catalogs. Deferred because the issue explicitly excludes price refresh and
  provider registry semantics.

## Risks

- Security: no secret handling changes; do not add network calls to cost calculation.
- Compatibility: legacy `core::cost` imports must continue compiling.
- Performance: default authority should use already-loaded bundled pricing and avoid per-call JSON parsing.
- Maintenance: provider alias matching must remain explicit so one provider does not borrow another provider's price row.

## Test Plan

- [ ] SpecRail: `python3 /Users/apple/Desktop/code/AI/tool/specrail/checks/check_workflow.py --repo /Users/apple/Desktop/code/AI/tool/specrail --spec-dir /private/tmp/litellm-rs-issue726/specs/GH726`
- [ ] Format: `cargo fmt --all -- --check`
- [ ] Targeted: `cargo test pricing --lib --locked`
- [ ] Targeted: `cargo test spend --lib --locked`
- [ ] Targeted: `cargo test runtime_pricing --lib --locked`
- [ ] Compile: `cargo check --all-features --locked`

## Rollback Plan

Revert the GH-726 PR. The slice should be limited to pricing service helpers, the legacy cost facade,
AI spend tests, pricing service tests, and the SpecRail packet. No persisted state or migration rollback is needed.
