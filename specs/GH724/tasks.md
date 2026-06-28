# Task Plan

## Linked Issue

GitHub issue: `#724`

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## Implementation Tasks

- [ ] `SP724-T001` Owner: `type-boundary-worker` | Done when: internal `ChatRequest` and OpenAI `ChatCompletionRequest` ownership is documented in code | Verify: `rg "Canonical internal chat request|OpenAI-compatible chat completion transport request" src/core/types/chat.rs src/core/models/openai/requests.rs`
- [ ] `SP724-T002` Owner: `type-boundary-worker` | Done when: `core::models::ModelInfo` resolves to canonical `core::types::model::ModelInfo` and the old shape is renamed `LegacyModelSummary` | Verify: `cargo test --lib test_core_models_model_info_uses_canonical_shape --locked`
- [ ] `SP724-T003` Owner: `route-boundary-worker` | Done when: `build_core_chat_request` preserves transport DTO fields at the internal request boundary | Verify: `cargo test --lib test_build_core_chat_request_preserves_transport_fields --locked`
- [ ] `SP724-T004` Owner: `openai-transformer-worker` | Done when: typed OpenAI request transformation forwards unknown provider extras without overriding typed fields | Verify: `cargo test --lib test_transform_request_forwards_extra_params_without_overriding_typed_fields --locked`
- [ ] `SP724-T005` Owner: `coordinator` | Done when: SpecRail, formatting, check, and full library tests pass | Verify: `python3 /Users/apple/Desktop/code/AI/tool/specrail/checks/check_workflow.py --repo /Users/apple/Desktop/code/AI/tool/specrail --spec-dir /private/tmp/litellm-rs-issue724/specs/GH724 && cargo fmt --all -- --check && cargo check --all-features --locked && cargo test --lib --locked`

## Parallelization

This tranche stays single-writer because the main edits touch shared type
definitions and route conversion tests. The threads explorer lane is read-only
and owns no writable files.

## Verification

- `python3 /Users/apple/Desktop/code/AI/tool/specrail/checks/check_workflow.py --repo /Users/apple/Desktop/code/AI/tool/specrail --spec-dir /private/tmp/litellm-rs-issue724/specs/GH724`
- `cargo fmt --all -- --check`
- `cargo check --all-features --locked`
- `cargo test --lib test_build_core_chat_request_preserves_transport_fields --locked`
- `cargo test --lib test_core_models_model_info_uses_canonical_shape --locked`
- `cargo test --lib test_transform_request_forwards_extra_params_without_overriding_typed_fields --locked`
- `cargo test --lib --locked`

## Handoff Notes

Do not use this issue to rewrite provider dispatch, provider registry, pricing,
or SDK/HTTP provider parity. Those are tracked by GH-725 through GH-729.
