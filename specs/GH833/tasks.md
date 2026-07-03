# Task Plan

## Linked Issue

GH-833 / #833

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## 实现任务

- [ ] `SP833-T1` Owner: coordinator. Done when: `specs/GH833/` 三件套通过 SpecRail packet validation. Verify: `SPEC_RAIL=/path/to/specrail; python3 "$SPEC_RAIL/checks/check_workflow.py" --repo "$SPEC_RAIL" --spec-dir "$PWD/specs/GH833"`.
- [ ] `SP833-T2` Owner: coordinator. Done when: rate-limit header facts helper 覆盖 `GatewayError::RateLimit` 与 `GatewayError::Provider(ProviderError::RateLimit)`. Verify: focused unit tests assert headers for both variants.
- [ ] `SP833-T3` Owner: coordinator. Done when: response renderer writes `Retry-After`, `X-RateLimit-Limit-Requests`, `X-RateLimit-Limit-Tokens` for provider 429 when metadata exists. Verify: `cargo test utils::error::gateway_error --lib --all-features` or exact module test.
- [ ] `SP833-T4` Owner: coordinator. Done when: A2A/MCP conversions preserve `retry_after_ms` as HTTP seconds with ceil/min-1 semantics. Verify: conversion unit tests for `1500 -> 2`, `1 -> 1`, `None -> None`.
- [ ] `SP833-T5` Owner: verification owner. Done when: #839 compatibility note added to tests or code comments so future mapping unification cannot drop headers. Verify: `rg -n "Retry-After|X-RateLimit-Limit|rate_limit_headers|http_facts" src/utils src/server`.
- [ ] `SP833-T6` Owner: verification owner. Done when: format/lint/tests pass. Verify: `cargo fmt --all -- --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-features`.

## 并行拆分

- T2/T3 and T4 touch different functions but same module family; keep in one small PR to avoid partial header behavior.

## 验证

- [ ] `SP833-T7` Owner: verification owner. Done when: PR body includes fresh test output for provider, gateway, A2A, and MCP rate-limit cases. Verify: paste command output from this session.

## Handoff Notes

- Do not stringify/parse retry metadata.
- Do not invent headers when metadata is absent.
- If #839 lands first, implement this in the canonical facts layer instead of adding a temporary duplicate table.
