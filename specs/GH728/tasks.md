# Task Plan

## Linked Issue

GH-728 / #728

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## 实现任务

- [x] `SP728-T1` Owner: coordinator. Done when: `specs/GH728/product.md`, `tech.md`, and `tasks.md` exist and pass SpecRail packet validation. Verify: `python3 /Users/apple/Desktop/code/AI/tool/specrail/checks/check_workflow.py --repo /Users/apple/Desktop/code/AI/tool/specrail --spec-dir /tmp/litellm-rs-issue728/specs/GH728`.
- [x] `SP728-T2` Owner: coordinator. Done when: registry exports a canonical support matrix for HTTP, SDK, and completion route surfaces. Verify: `cargo test support_matrix --lib`.
- [x] `SP728-T3` Owner: coordinator. Done when: SDK chat/stream/embeddings routing and direct execution use matrix-backed support checks. Verify: focused SDK routing/execution tests.
- [x] `SP728-T4` Owner: coordinator. Done when: `completion()` reports explicit unsupported errors for known unsupported provider prefixes. Verify: focused completion router test.
- [x] `SP728-T5` Owner: docs. Done when: README documents the cross-surface support matrix. Verify: registry README tests and manual diff review.
- [ ] `SP728-T6` Owner: verification owner. Done when: formatting, focused tests, SpecRail packet validation, all-features check, PR CI, and review-thread gate pass from this session. Verify: `cargo fmt --all -- --check`; focused cargo tests; `cargo check --all-features --locked`; GitHub PR CI and review-thread query.

## 并行拆分

#728 is a serial writable lane after #729. Do not edit #727 U-16 split files from this lane except where a focused #728 test touches an already oversized file. #727 remains the dedicated file-size tranche.

Writable ownership for this lane:

- `specs/GH728/`
- `src/core/providers/registry/support_matrix.rs`
- `src/core/providers/registry/mod.rs`
- `src/sdk/client/routing.rs`
- `src/sdk/client/completions.rs`
- `src/sdk/client/embeddings.rs`
- `src/sdk/client/tests.rs`
- `src/core/completion/default_router/router_impl.rs`
- `README.md`

## 验证

- SpecRail packet validation.
- `cargo fmt --all -- --check`
- Focused SDK, support matrix, and completion router tests.
- `cargo check --all-features --locked`
- PR head SHA, CI, merge state, and GraphQL review threads checked before merge.

## Handoff Notes

This issue intentionally marks SDK Google/Gemini chat unsupported instead of
implementing it. A future adapter PR can flip the matrix row only after adding
real Google request/response handling and tests.
