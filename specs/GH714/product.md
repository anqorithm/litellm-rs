# Product Spec

## Linked Issue

GH-714

## User Problem

Provider support is described in several places: the registry table,
`ProviderType`, factory branches, catalog entries, enum dispatch, and the README
matrix. When these drift, maintainers can document a provider that is not
constructible, or keep code wired without the public matrix reflecting it.

## Goals

- Treat the existing `ProviderRegistryEntry` table plus Tier 1 catalog as the
  canonical declaration source for this slice.
- Add a conformance guard that validates the README provider-support matrix
  against the canonical registry/catalog data.
- Ensure documented provider selectors are known to the registry or Tier 1
  catalog.
- Ensure registry dispatchability, factory support, and README support claims
  fail tests when they drift.
- Keep #714 separate from provider retry/error policy (#715) and the broad
  architecture roadmap (#519).

## Non-Goals

- Do not introduce code generation in this PR.
- Do not rewrite `ProviderType`, the `Provider` enum, dispatch macros, or
  factory construction branches.
- Do not add custom provider routing; #713 selected the closed built-in provider
  contract.
- Do not change endpoint capability semantics beyond matrix consistency.

## User-Visible Behavior

The runtime provider set should not change. The README matrix becomes a tested
claim: if a provider is advertised as supported, conformance tests must be able
to trace that selector to the registry/catalog/factory surface.

## Acceptance Criteria

- [x] `specs/GH714/` records the #714 validation-harness scope.
- [x] README provider matrix selectors are validated against registry/catalog
  declarations.
- [x] README no longer claims a provider construction path that disagrees with
  the registry/factory behavior.
- [x] Existing registry/factory conformance tests still pass.
- [x] #715 retry/error mapping and #519 type-tree refactors are untouched.

## Follow-Up

A later #714 slice can replace the hand-maintained README matrix with generated
docs once the validation harness is stable. #519 can still track broader type
taxonomy collapse.
