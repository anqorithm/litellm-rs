# Task Plan

## Linked Issue

GH-725 / #725

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## Implementation Tasks

- [x] `SP725-T001` Owner: registry-worker | Done when: registry exposes a catalog-dispatch helper and tests keep factory support equal to dispatchable registry entries. | Verify: cargo test provider_registry --lib
- [x] `SP725-T002` Owner: factory-worker | Done when: catalog provider construction is gated by `ProviderDispatchKind::CatalogOpenAiLike` instead of catalog presence alone. | Verify: cargo test from_config_async --lib
- [x] `SP725-T003` Owner: runtime-worker | Done when: `ProviderRegistry` tests prove canonical registration keys and explicit logical keys do not drift from provider identity. | Verify: cargo test provider_registry --lib
- [x] `SP725-T004` Owner: router-worker | Done when: default router catalog startup registration uses one catalog-driven list without repeated per-provider blocks. | Verify: cargo test provider_registry --lib
- [x] `SP725-T005` Owner: verifier | Done when: local SpecRail, format, targeted tests, and full feature compile pass. | Verify: cargo check --all-features --locked

## Parallelization

The coordinator owns all writable files for this tranche to avoid overlapping
provider registry/factory edits. The read-only explorer lane inspected the same
areas and returned a plan but did not edit files.

## Verification

- SpecRail: `python3 /Users/apple/Desktop/code/AI/tool/specrail/checks/check_workflow.py --repo /Users/apple/Desktop/code/AI/tool/specrail --spec-dir /private/tmp/litellm-rs-issue725/specs/GH725`
- Format: `cargo fmt --all -- --check`
- Targeted: `cargo test provider_registry --lib`
- Targeted: `cargo test from_config_async --lib`
- Compile: `cargo check --all-features --locked`

## Handoff Notes

Do not claim GH-725 removes all manual enum dispatch. The accepted slice makes
the existing manual surfaces mechanically guarded and moves catalog factory/runtime
paths behind registry metadata. Full enum macro generation should be a separate
refactor if still needed after GH-725.
