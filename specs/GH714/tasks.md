# Task Plan

## Linked Issue

GH-714

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## Implementation Tasks

- [x] `SP714-T1` Owner: coordinator. Done when: SpecRail product and tech specs record the validation-harness slice and #715/#519 boundaries. Verify: `check_workflow.py --spec-dir /private/tmp/litellm-rs-issue714/specs/GH714`.
- [x] `SP714-T2` Owner: coordinator. Done when: README provider support wording reflects registry/catalog validation rather than unguarded manual claims. Verify: README diff and matrix guard.
- [x] `SP714-T3` Owner: coordinator. Done when: README Tier 1/Tier 2/experimental provider selectors are checked against registry/catalog declarations. Verify: `cargo test provider_registry --lib`.
- [x] `SP714-T4` Owner: coordinator. Done when: existing factory/dispatch conformance remains green. Verify: `cargo test test_dispatch_kind_matches_runtime_variant --lib --all-features`.
- [x] `SP714-T5` Owner: coordinator. Done when: PR body records SpecRail gates, threads closure audit, CI, and reviewThreads. Verify: PR #722 body includes the merge gate checklist for CI, GraphQL reviewThreads, and SpecRail PR evidence.

## Parallel Split

- Main coordinator owns writable files for `specs/GH714/**`, README, and
  provider registry tests.
- Threads lane #714 is read-only planning/review of registry/factory/docs
  conformance.
- No writable lane may touch #715 retry/error mapping or broad #519
  architecture refactors.

## Verification

- `python3 /Users/apple/Desktop/code/AI/tool/specrail/checks/route_gate.py --repo /Users/apple/Desktop/code/AI/tool/specrail --route write_spec --issue 714 --state ready_to_spec --mode advisory --json`
- `python3 /Users/apple/Desktop/code/AI/tool/specrail/checks/check_workflow.py --repo /Users/apple/Desktop/code/AI/tool/specrail --spec-dir /private/tmp/litellm-rs-issue714/specs/GH714`
- `cargo test provider_registry --lib`
- `cargo test test_dispatch_kind_matches_runtime_variant --lib --all-features`
- `cargo check --all-features --locked`

## Handoff Notes

The current PR should provide a guardrail, not a code-generation rewrite. Later
#714 slices can generate docs from the registry once this validation is stable.
