# Task Plan

## Linked Issue

GH-1105 / #1105

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## Implementation Tasks

- [ ] `SP1105-T1` Covers: B-001, B-002, B-003, B-004, B-005, B-006, B-007, B-008. Owner: coordinator. Dependencies: none. Done when: product/tech/tasks exist, behavior invariants are contiguous, and every invariant maps to implementation and tests. Verify: SpecRail workflow checks and `git diff --check`.
- [ ] `SP1105-T2` Covers: B-003, B-004, B-006, B-007, B-008. Owner: implementation owner. Dependencies: SP1105-T1. Done when: gateway/provider schema, defaults, merge, debug, and deterministic alias validation are implemented with negative cases. Verify: focused config model and validator tests.
- [ ] `SP1105-T3` Covers: B-001, B-004, B-005, B-006, B-007. Owner: implementation owner. Dependencies: SP1105-T2. Done when: router construction atomically installs validated aliases and propagates priority without breaking the existing public constructor. Verify: focused gateway-config and priority-routing tests.
- [ ] `SP1105-T4` Covers: B-002. Owner: implementation owner. Dependencies: SP1105-T3. Done when: alias names are read from router state and appear once in deterministic OpenAI model inventory output. Verify: focused model-route tests.
- [ ] `SP1105-T5` Covers: B-001, B-002, B-005, B-006. Owner: documentation owner. Dependencies: SP1105-T2. Done when: example YAML and README document alias, primary/fallback priority, defaults, and rollback ordering. Verify: example config parse test and documentation diff review.
- [ ] `SP1105-T6` Covers: B-001, B-002, B-003, B-004, B-005, B-006, B-007, B-008. Owner: verification owner. Dependencies: SP1105-T3, SP1105-T4, SP1105-T5. Done when: focused and repository-wide gates pass and the diff remains scoped to GH1105. Verify: `cargo fmt --all -- --check`; `cargo check --all-targets --all-features --locked`; `cargo clippy --all-targets --all-features --locked -- -D warnings`; `cargo test --all-features --locked -- --test-threads=1`; workflow/spec checks; `git diff --check`.

## Parallelization

Schema/validation and docs share example expectations and should follow the approved spec serially. Model inventory work can proceed after the router accessor contract is fixed. One implementation owner should handle the tightly coupled config and router files; verification remains independent.

## Verification

- Run `python3 checks/check_workflow.py --repo .`.
- Run `python3 checks/check_workflow.py --repo . --spec-dir=specs/GH1105`.
- Confirm the product invariant set and task `Covers:` union both equal B-001 through B-008.
- Require the implementation PR to use `Fixes #1105`, target the spec branch, and report only current-head CI evidence.

## Handoff Notes

- Preserve `Router::from_gateway_config` for external callers; use an additive constructor/helper for aliases.
- Alias validation must be independent of `HashMap` iteration order.
- Lower numeric priority is selected first; equal-priority behavior remains unchanged.
- Do not merge or give final approval automatically; repository policy reserves those gates for a human.
