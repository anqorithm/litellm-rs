# Tech Spec

## Linked Issue

GH-727 / #727

## Product Spec

Link to `product.md`.

## Current Evidence

At `origin/main@56f8cb2a`, 5 tracked Rust files remain over the U-16
800-line ceiling. The current largest file is `src/core/providers/openai/client_tests.rs`
at 809 lines. It is a test-only OpenAI provider suite covering provider creation,
properties, model support, supported params, request transform, OpenAI-like passthrough,
cost calculation, error mapping, request headers, clone/debug, and convenience helpers.

## Architecture Principles

1. Facade compatibility: when splitting public type files, the original module path keeps
   re-exporting the same public names with `pub use`.
2. Runtime ownership: runtime files split by one responsibility at a time, such as request
   mapping, response mapping, operation handlers, storage helpers, or error conversion.
3. Test-suite ownership: test-only content moves by behavior domain while keeping original
   assertions and focused test command coverage.
4. Minimal public-type churn: if a public type file is only oversized because of inline
   tests, extract tests first instead of inventing a facade hierarchy.
5. No silent degradation: moved code must preserve current error propagation and must not
   add warning-only fallbacks for previously failing states.
6. Bounded PRs: each tranche owns one file family and includes line-count proof plus focused
   tests.

## Queue Design

| Phase | Lane | Target examples | Verification pattern |
| --- | --- | --- | --- |
| P1 | Test suites | DataUtils tests, router tests, utils/event tests, provider test files, integration route tests | focused `cargo test` for the moved module plus line-count proof |
| P2 | Public type facades | SDK, analytics, monitoring, observability, audio, config, user/key/cache types | choose test extraction when tests cause the oversize; otherwise use facade + `pub use` |
| P3 | Runtime orchestrators | OpenTelemetry, OAuth session, request validator | focused module tests or affected integration tests plus all-features check |
| P4 | Utility modules | config helpers, net client utils, sync containers | focused utility tests plus concurrency/behavior checks where relevant |
| P5 | Closure scan | all Rust files | full over-800 scan, final SpecRail update, final PR may close #727 |

## Current Tranche: OpenAI Client Test-Suite Split

### Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| Test facade | `src/core/providers/openai/client_tests.rs` | Currently contains shared helper factories plus all OpenAI provider behavior tests directly. | This file can keep shared test setup while delegating behavior tests. |
| Extracted child tests | `src/core/providers/openai/client_tests/*.rs` | New behavior-domain test modules under the existing `openai::client_tests` facade. | Splitting by provider behavior reduces file size without changing runtime code. |
| Existing siblings | `streaming_request_tests.rs`, `transformer/response_tests.rs` | Other OpenAI-focused test modules. | They remain untouched to keep ownership boundaries clear. |

### Design

1. Keep `src/core/providers/openai/client_tests.rs` as the test facade with the original imports, `create_test_config`, `create_test_provider`, typed-param request helper, and typed-param assertion helper.
2. Split the original tests into child modules under `src/core/providers/openai/client_tests/`:
   - `provider_support_tests.rs` for provider creation/properties, model support, model info/config, and supported params.
   - `request_transform_tests.rs` for chat request transform, typed-param forwarding, OpenAI-like passthrough, and cost calculation.
   - `error_header_tests.rs` for error mapper, request headers, clone, and debug tests.
   - `convenience_tests.rs` for model recommendations, feature support, pricing, and context window helpers.
3. Each child module uses `use super::*;` to retain the same access to shared provider helper factories.
4. Move tests without assertion, fixture API key/model, JSON expected-value, pricing, context, or error-message expectation changes.
5. Do not edit production OpenAI provider, client, registry, transformer, streaming, cost source, or OpenAI-like provider code.

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P1 | `src/core/providers/openai/client_tests.rs` | Root test facade keeps shared helpers and delegates OpenAI provider tests to child modules. |
| P2 | `src/core/providers/openai/client_tests/*.rs` | Original test names and assertions remain present under behavior-domain modules. |
| P3 | OpenAI provider behavior | No provider creation, properties, model support, supported params, request transform, cost, error mapper, header, clone/debug, recommendation, pricing, or context behavior changes. |
| P4 | file size | `wc -l src/core/providers/openai/client_tests.rs src/core/providers/openai/client_tests/*.rs` shows every touched file below 800. |
| P5 | focused test suite | `cargo test core::providers::openai::client_tests --lib --all-features` runs the moved tests. |
| P6 | queue count | tracked-file scan shows the remaining queue no longer includes `src/core/providers/openai/client_tests.rs`. |

## Risks

- Splitting a provider test file changes test module paths below `openai::client_tests`, so focused filtering should use `core::providers::openai::client_tests`.
- Child modules must remain under the `client_tests.rs` facade so they share provider helper factories through `super::*`.
- OpenAI provider behavior is customer-facing provider behavior; this tranche must not modify production provider, registry, transformer, cost, or OpenAI-like code, and must not weaken assertions.

## Test Plan

- [ ] `cargo fmt --all -- --check`
- [ ] `git diff --check`
- [ ] `python3 /Users/apple/Desktop/code/AI/tool/specrail/checks/check_workflow.py --repo /Users/apple/Desktop/code/AI/tool/specrail --spec-dir "$PWD/specs/GH727"`
- [ ] `wc -l src/core/providers/openai/client_tests.rs src/core/providers/openai/client_tests/*.rs`
- [ ] `cargo test core::providers::openai::client_tests --lib --all-features`
- [ ] `cargo check --lib --all-features`
- [ ] `cargo check --all-features --locked`
- [ ] `cargo check`

## Rollback

Move the OpenAI provider test modules back into `src/core/providers/openai/client_tests.rs`
and revert the `specs/GH727` edits. No schema, persistence, or runtime behavior
changes are involved.
