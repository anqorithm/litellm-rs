# Task Plan

## Linked Issue

GH-727 / #727

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## 实现任务

- [x] `SP727-T1` Owner: coordinator. Done when: `specs/GH727/product.md`, `tech.md`, and `tasks.md` exist and pass SpecRail packet validation. Verify: from this repository with a local SpecRail checkout, `SPEC_RAIL=/path/to/specrail; python3 "$SPEC_RAIL/checks/check_workflow.py" --repo "$SPEC_RAIL" --spec-dir "$PWD/specs/GH727"`.
- [x] `SP727-T2` Owner: coordinator. Done when: `src/core/providers/thinking/tests.rs` is split into `tests/mod.rs` and provider-specific child modules. Verify: `git diff --stat`; `wc -l src/core/providers/thinking/tests/*.rs`.
- [x] `SP727-T3` Owner: coordinator. Done when: moved tests compile and pass with unchanged assertions. Verify: `cargo test core::providers::thinking --lib`.
- [ ] `SP727-T4` Owner: verification owner. Done when: formatting, all-features check, PR CI, and review-thread gate pass. Verify: `cargo fmt --all -- --check`; `cargo check --all-features --locked`; GitHub PR CI and review-thread query.

## 并行拆分

This is a serial writable lane for one test file family. Other #727 large-file tranches may be planned read-only in parallel, but they must not edit this branch.

Writable ownership for this lane:

- `specs/GH727/`
- `src/core/providers/thinking/tests.rs`
- `src/core/providers/thinking/tests/`

## 验证

- SpecRail packet validation.
- `cargo fmt --all -- --check`
- `cargo test core::providers::thinking --lib`
- `cargo check --all-features --locked`
- PR CI and GraphQL review-thread gate before merge.

## Handoff Notes

This PR is the first #727 maintenance tranche and should not use `Closes #727`.
The issue should remain open until enough large-file tranches are completed or the tracker is explicitly closed.
