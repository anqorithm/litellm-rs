# Task Plan

## Linked Issue

GH-713

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## Implementation Tasks

- [x] `SP713-T1` Owner: coordinator. Done when: SpecRail product and tech specs record the closed built-in provider contract and #714 boundary. Verify: `check_workflow.py --spec-dir /private/tmp/litellm-rs-issue713/specs/GH713`.
- [x] `SP713-T2` Owner: coordinator. Done when: `LLMProvider`, `Provider`, `Deployment`, and `ProviderHandle` docs no longer imply standalone custom-provider router support. Verify: targeted `rg` for provider-contract wording.
- [x] `SP713-T3` Owner: coordinator. Done when: `ProviderHandle` stops returning optimistic support, health, cost, latency, and success-rate data. Verify: `cargo test provider_handle --lib`.
- [x] `SP713-T4` Owner: coordinator. Done when: #713 remains separate from #714 registry/source-of-truth implementation. Verify: git diff contains no registry/catalog/provider matrix rewrite.
- [ ] `SP713-T5` Owner: coordinator. Done when: PR body records SpecRail readiness/review/merge gates and threads closure audit. Verify: PR template checklist is completed with fresh CI/review evidence.

## Parallel Split

- Main coordinator owns writable files for `specs/GH713/**`,
  `src/core/traits/provider/**`, `src/core/providers/mod.rs`, and
  `src/core/router/deployment.rs`.
- Threads lane #713 is read-only planning/review of the provider contract.
- Threads lane #714 is read-only planning so registry/source-of-truth work stays
  out of this PR.

## Verification

- `python3 /Users/apple/Desktop/code/AI/tool/specrail/checks/route_gate.py --repo /Users/apple/Desktop/code/AI/tool/specrail --route write_spec --issue 713 --state ready_to_spec --mode advisory --json`
- `python3 /Users/apple/Desktop/code/AI/tool/specrail/checks/check_workflow.py --repo /Users/apple/Desktop/code/AI/tool/specrail --spec-dir /private/tmp/litellm-rs-issue713/specs/GH713`
- `cargo test provider_handle --lib`
- `cargo test test_create_provider_reports_unknown_custom_provider --lib`
- `cargo test provider_registry_contains_all_non_custom_provider_types --lib`
- `cargo check --all-features --locked`
- `cargo test --lib`

## Handoff Notes

SpecRail implementation is advisory until maintainer labels are applied on
GitHub. The user has authorized continuing through PR and merge gates when
threads, CI, reviewThreads, and SpecRail review evidence are clean.
