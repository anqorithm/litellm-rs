# Tech Spec

## Linked Issue

GH-714

## Product Spec

Link to `product.md`.

## Current System

- `src/core/providers/registry/types.rs` already defines
  `PROVIDER_TYPE_REGISTRY` with canonical provider names, aliases,
  dispatch kind, and catalog flags.
- `Provider::factory_supported_provider_types()` is backed by
  `registry::dispatchable_provider_types_slice()`.
- Existing tests verify registry coverage for non-custom `ProviderType`
  variants, dispatchability versus factory support, native enum variants, and
  catalog flags.
- `src/core/providers/factory/registry.rs` has a runtime-shape test that checks
  dispatchable registry entries create the expected `Provider` variant.
- README still describes the provider matrix as hand-maintained, and it can
  drift from registry/factory semantics.

## Proposed Design

1. Keep `PROVIDER_TYPE_REGISTRY` as the canonical source for this PR.
2. Add a README matrix conformance test in
   `src/core/providers/registry/readme_tests.rs`:
   - Parse Tier 2 provider selectors from the first code span of each provider
     matrix row's Provider cell.
   - Parse the Cargo feature cell and compare it to a cfg-independent expected
     Tier 2 selector/feature-gate set.
   - Parse Tier 1 catalog selectors from the cloud/local code-list lines.
   - Parse experimental/module-only selectors from the experimental section.
   - Assert Tier 2 selectors exist in the registry and cannot disappear under
     default feature tests when they are constructible under optional features.
   - Assert Tier 1 selectors exist in `PROVIDER_CATALOG`.
   - Assert every catalog selector is documented in the README provider support
     section.
   - Assert every dispatchable registry selector is documented.
   - Assert experimental/module-only selectors are not catalog entries,
     dispatchable entries, or expected Tier 2 provider-support rows.
3. Update the README provider support note from "hand-maintained only" to
   "validated against registry/catalog".
4. Correct any README row that the new guard exposes as inconsistent with the
   current registry/factory behavior.

## Alternatives

- Generate README from Rust metadata now: better long term, but broader tooling
  and formatting churn.
- Move all provider declarations into a macro/table that generates enum/factory
  branches: valuable, but it crosses into the larger #519 architecture work.
- Only add factory tests: existing tests already cover much of this; the missing
  surface is documentation drift.

## Test Plan

- [x] Unit tests: README matrix guard in `provider_registry`.
- [x] Existing unit tests: `provider_registry` and factory dispatch-kind guards.
- [x] `cargo test provider_registry --lib`.
- [x] `cargo test test_dispatch_kind_matches_runtime_variant --lib --all-features`.
- [x] `cargo check --all-features --locked`.

## Rollback Plan

If the README parser is too brittle, revert the new docs guard while keeping any
actual README correction. No runtime behavior or persisted data changes are
involved.
