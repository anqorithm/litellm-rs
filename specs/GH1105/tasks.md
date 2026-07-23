# Task Plan

## Linked Issue

GH-1105 / #1105

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## Implementation Tasks

- [ ] `SP1105-T1` Covers: B-001, B-002, B-003, B-004, B-005, B-006, B-007, B-008. Owner: coordinator. Dependencies: none. Done when: product/tech/tasks exist, behavior invariants are contiguous, and every invariant maps to implementation and tests. Verify: SpecRail workflow checks and `git diff --check`.
- [ ] `SP1105-T2` Covers: B-003, B-004, B-006, B-007, B-008. Owner: implementation owner. Dependencies: SP1105-T1. Done when: gateway/provider schema and defaults exist; alias overlays merge key by key with overlay wins and omitted/explicit-empty overlays preserving base entries; Phase A rejects empty/self/cyclic graphs without checking provider-expanded targets. Verify: focused config model merge/default/export tests and graph-validator negative tests.
- [ ] `SP1105-T3` Covers: B-001, B-002, B-004, B-005, B-006, B-007. Owner: implementation owner. Dependencies: SP1105-T2. Done when: router construction expands enabled configured/dynamic/fallback canonical models, rejects alias-key collisions and absent final targets, flattens every chain to one hop, and publishes the complete deployment/alias state before health checks while propagating priority and preserving the existing public constructor. Verify: focused gateway-config tests for dynamic targets, disabled/missing targets, canonical collision, reversed declaration order, a chain longer than 16 hops, atomic failure, and priority routing.
- [ ] `SP1105-T4` Covers: B-002. Owner: implementation owner. Dependencies: SP1105-T3. Done when: alias names are read from router state and appear once in deterministic OpenAI model inventory output. Verify: focused model-route tests.
- [ ] `SP1105-T5` Covers: B-001, B-002, B-005, B-006. Owner: documentation owner. Dependencies: SP1105-T2. Done when: example YAML and README document alias, primary/fallback priority, defaults, and rollback ordering. Verify: example config parse test and documentation diff review.
- [ ] `SP1105-T6` Covers: B-001, B-002, B-003, B-004, B-005, B-006, B-007, B-008. Owner: verification owner. Dependencies: SP1105-T3, SP1105-T4, SP1105-T5. Done when: focused and repository-wide gates pass and the diff remains scoped to GH1105. Verify: `cargo fmt --all -- --check`; `cargo check --all-targets --all-features --locked`; `cargo clippy --all-targets --all-features --locked -- -D warnings`; `cargo test --all-features --locked -- --test-threads=1`; workflow/spec checks; `git diff --check`.

## Parallelization

Schema/Phase-A validation and docs share example expectations and should follow the approved spec serially. Phase-B expansion/flattening owns router construction and must finish before model inventory work. One implementation owner should handle the tightly coupled config and router files; verification remains independent.

## Verification

- Run `python3 checks/check_workflow.py --repo .`.
- Run `python3 checks/check_workflow.py --repo . --spec-dir=specs/GH1105`.
- Confirm the product invariant set and task `Covers:` union both equal B-001 through B-008.
- Require the implementation PR to use `Fixes #1105`, target the spec branch, and report only current-head CI evidence.

## Handoff Notes

- Preserve `Router::from_gateway_config` for external callers; use an additive constructor/helper for aliases.
- Alias graph validation and flattening must be independent of `HashMap` iteration order.
- Treat omitted and explicit-empty overlay alias maps identically: both preserve base aliases; this version has no clearing sentinel.
- Derive canonical-model collisions and final-target validity from staged enabled deployments after dynamic model expansion, not only from YAML model lists.
- Install only single-hop alias-to-canonical mappings so runtime resolution never depends on the 16-hop bound.
- Lower numeric priority is selected first; equal-priority behavior remains unchanged.
- Do not merge or give final approval automatically; repository policy reserves those gates for a human.
