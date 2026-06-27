# Task Plan

## Linked Issue

GH-715

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## Implementation Tasks

- [x] `SP715-T1` Owner: coordinator. Done when: SpecRail product and tech docs record the failure/retry/HTTP split and #519 boundary. Verify: `check_workflow.py --spec-dir /private/tmp/litellm-rs-issue715/specs/GH715`.
- [x] `SP715-T2` Owner: coordinator. Done when: `ProviderFailureFacts` exposes provider failure facts and retry hints without policy methods. Verify: `cargo test failure --lib`.
- [x] `SP715-T3` Owner: coordinator. Done when: `RetryPolicy` decides from context and preserves selected-deployment budget fallback behavior. Verify: `cargo test retry_policy --lib` and `cargo test budget_retry_fallbacks_skip_retry_delay --lib`.
- [x] `SP715-T4` Owner: coordinator. Done when: server/router execution uses `RetryPolicy` for unary and pre-output streaming retries. Verify: `cargo test execute_with_selected_deployment --lib`.
- [x] `SP715-T5` Owner: coordinator. Done when: OpenAI HTTP mapping remains at adapter boundary. Verify: `cargo test provider_timeout_http_mapping_lives_at_openai_adapter_boundary --lib`.

## Parallel Split

- Main coordinator owns writable files for `specs/GH715/**`, provider failure
  facts, router retry policy, and selected-deployment execution.
- Threads planner lane is read-only and may inspect retry/error/HTTP surfaces.
- Merge-reviewer lane is read-only after PR creation.
- No writable lane may touch #519 trait/type-tree refactors.

## Verification

- `python3 /Users/apple/Desktop/code/AI/tool/specrail/checks/check_workflow.py --repo /Users/apple/Desktop/code/AI/tool/specrail --spec-dir /private/tmp/litellm-rs-issue715/specs/GH715`
- `cargo test failure --lib`
- `cargo test retry_policy --lib`
- `cargo test budget_retry_fallbacks_skip_retry_delay --lib`
- `cargo test execute_with_selected_deployment --lib`
- `cargo test core::router::tests::execution_tests --lib`
- `cargo test provider_timeout_http_mapping_lives_at_openai_adapter_boundary --lib`
- `cargo check --all-features --locked`

## Handoff Notes

This is a migration slice. Removing `ProviderError` compatibility helpers is
deliberately deferred until callers have switched to policy and adapter
surfaces.
