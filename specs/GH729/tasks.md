# Task Plan

## Linked Issue

GH-729 / #729

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## 实现任务

- [x] `SP729-T1` Owner: coordinator. Done when: `specs/GH729/product.md`, `tech.md`, and `tasks.md` exist and pass SpecRail packet validation. Verify: `python3 /Users/apple/Desktop/code/AI/tool/specrail/checks/check_workflow.py --repo /Users/apple/Desktop/code/AI/tool/specrail --spec-dir /tmp/litellm-rs-issue729/specs/GH729`.
- [x] `SP729-T2` Owner: coordinator. Done when: `LLMProvider` exposes the canonical `supports_capability()` predicate and optional helper methods delegate to it. Verify: focused provider trait tests.
- [x] `SP729-T3` Owner: coordinator. Done when: `Provider` and router/server capability scans use the shared predicate instead of duplicated slice scans. Verify: focused router/provider tests.
- [x] `SP729-T4` Owner: coordinator. Done when: `sub_traits.rs` clearly documents legacy compatibility/migration guidance while keeping deprecated symbols available. Verify: focused sub-trait tests compile and pass.
- [x] `SP729-T5` Owner: verification owner. Done when: formatting, focused tests, SpecRail packet validation, and all-features check pass from this session. Verify: `cargo fmt --all -- --check`; `cargo test provider --lib`; `cargo check --all-features --locked`.

## 并行拆分

#729 is a serial writable lane. `#728` may plan read-only in parallel but must not edit SDK/support-matrix files until this branch is merged or rebased. `#727` may plan read-only in parallel and should choose files outside provider trait/router capability ownership.

Writable ownership for this lane:

- `specs/GH729/`
- `src/core/traits/provider/llm_provider/`
- `src/core/providers/mod.rs`
- `src/core/router/selection.rs`
- `src/core/router/unified.rs`
- `src/server/routes/ai/provider_selection.rs`

## 验证

- SpecRail packet validation.
- `cargo fmt --all -- --check`
- `cargo test provider --lib`
- `cargo check --all-features --locked`
- PR head SHA, CI, merge state, and GraphQL review threads checked before merge.

## Handoff Notes

This issue intentionally chooses `ProviderCapability` as the runtime dispatch contract. Do not reopen the sub-trait carve-out in `#728`; the support matrix should consume this capability predicate and provider registry truth.
