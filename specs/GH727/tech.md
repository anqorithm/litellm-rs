# Tech Spec

## Linked Issue

GH-727 / #727

## Product Spec

Link to `product.md`.

## Current Evidence

At `origin/main@804aff95`, 4 tracked Rust files remain over the U-16
800-line ceiling. The current largest file is `src/core/observability/metrics.rs`
at 808 lines. It is a production metrics collector module whose production code ends
before the inline test module; the oversize is caused by unit tests for Prometheus metrics,
collector configuration, request/cache/provider-health recording, Prometheus export,
DataDog send behavior, duration recording, and edge cases.

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

## Current Tranche: Observability Metrics Test Extraction

### Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| Production metrics module | `src/core/observability/metrics.rs` | Defines `PrometheusMetrics`, `DataDogClient`, `OtelExporter`, and `MetricsCollector`, plus recording/export methods. | These public structs and methods stay in place to preserve re-export compatibility. |
| Extracted unit tests | `src/core/observability/metrics_tests.rs` | New path-backed child test module under `metrics.rs`. | Keeps tests close to the private fields they assert while reducing the production file size. |
| Observability siblings | `histogram.rs`, `types.rs`, `mod.rs` | Histogram storage, `TokenUsage`, and public re-exports. | They remain untouched to keep the tranche limited to metrics tests. |

### Design

1. Keep all production definitions and methods in `src/core/observability/metrics.rs`.
2. Replace the inline `#[cfg(test)] mod tests { ... }` body with:
   - `#[cfg(test)]`
   - `#[path = "metrics_tests.rs"]`
   - `mod tests;`
3. Move the original test body into `src/core/observability/metrics_tests.rs` with `use super::*;`.
4. Move tests without assertion, metric name, label, cache counter, provider-health value, token/cost counter, histogram, or DataDog no-op expectation changes.
5. Do not edit `src/core/observability/mod.rs`, histogram storage, `TokenUsage`, HTTP client setup, or unrelated observability modules.

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P1 | `src/core/observability/metrics.rs` | Production metrics structs and methods remain in the original module with path-backed test delegation. |
| P2 | `src/core/observability/metrics_tests.rs` | Original test names and assertions remain present in the child test module. |
| P3 | observability metrics behavior | No request/cache/provider-health recording, Prometheus export, DataDog send, duration, token, cost, or edge-case behavior changes. |
| P4 | file size | `wc -l src/core/observability/metrics.rs src/core/observability/metrics_tests.rs` shows both files below 800. |
| P5 | focused test suite | `cargo test core::observability::metrics --lib --all-features` runs the moved tests. |
| P6 | queue count | tracked-file scan shows the remaining queue no longer includes `src/core/observability/metrics.rs`. |

## Risks

- Tests assert private `datadog_client` and `otel_exporter` fields, so the extracted file must remain a child module of `metrics.rs`, not a sibling top-level module.
- The focused test filter should use `core::observability::metrics` because the test module remains nested below the metrics module.
- Metrics behavior is observability-critical; this tranche must not modify metric names, labels, counters, histogram behavior, export formatting, or error/no-op behavior.

## Test Plan

- [ ] `cargo fmt --all -- --check`
- [ ] `git diff --check`
- [ ] `python3 /Users/apple/Desktop/code/AI/tool/specrail/checks/check_workflow.py --repo /Users/apple/Desktop/code/AI/tool/specrail --spec-dir "$PWD/specs/GH727"`
- [ ] `wc -l src/core/observability/metrics.rs src/core/observability/metrics_tests.rs`
- [ ] `cargo test core::observability::metrics --lib --all-features`
- [ ] `cargo check --lib --all-features`
- [ ] `cargo check --all-features --locked`
- [ ] `cargo check`

## Rollback

Move the observability metrics tests back into `src/core/observability/metrics.rs`
and revert the `specs/GH727` edits. No schema, persistence, or runtime behavior
changes are involved.
