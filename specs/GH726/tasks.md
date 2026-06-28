# Task Plan

## Linked Issue

GH-726 / #726

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## Implementation Tasks

- [x] `SP726-T001` Owner: pricing-service-worker | Dependencies: none | Done when: `PricingService` exposes provider-aware lookup/calculation helpers for already-loaded pricing data and preserves fail-closed missing-pricing behavior. | Verify: `cargo test pricing_service --lib --locked`
- [x] `SP726-T002` Owner: cost-adapter-worker | Dependencies: `SP726-T001` | Done when: `generic_cost_per_token`, `estimate_cost`, and `get_model_pricing` use the PricingService-backed authority and only map into legacy DTOs. | Verify: `cargo test estimate_cost --lib --locked`
- [x] `SP726-T003` Owner: spend-worker | Dependencies: `SP726-T002` | Done when: AI spend reservation and completion settlement are covered by tests proving the same authority-backed cost values are used. | Verify: `cargo test spend --lib --locked`
- [x] `SP726-T004` Owner: provider-pricing-worker | Dependencies: `SP726-T002` | Done when: at least one provider-specific alias/pricing case proves compatibility through the new authority path. | Verify: `cargo test runtime_pricing --lib --locked`
- [x] `SP726-T005` Owner: verifier | Dependencies: `SP726-T001`, `SP726-T002`, `SP726-T003`, `SP726-T004` | Done when: SpecRail check, formatting, focused tests, and all-features compile pass. | Verify: `cargo check --all-features --locked`

## Parallelization

The coordinator owns writable files for this tranche because `PricingService`,
the legacy cost facade, and AI spend tests share a dependency chain. The native
threads lane for GH-726 is read-only architecture exploration only.

## Verification

- SpecRail: `python3 /Users/apple/Desktop/code/AI/tool/specrail/checks/check_workflow.py --repo /Users/apple/Desktop/code/AI/tool/specrail --spec-dir /private/tmp/litellm-rs-issue726/specs/GH726`
- Format: `cargo fmt --all -- --check`
- Targeted: `cargo test pricing_service --lib --locked`
- Targeted: `cargo test estimate_cost --lib --locked`
- Targeted: `cargo test spend --lib --locked`
- Targeted: `cargo test runtime_pricing --lib --locked`
- Compile: `cargo check --all-features --locked`

## Handoff Notes

Do not overclaim that all pricing-like structs are removed. In this slice, legacy
`core::cost` DTOs may remain as compatibility data shapes, but they must no longer
own an independent user-visible pricing calculation source.
