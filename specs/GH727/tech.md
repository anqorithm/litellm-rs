# Tech Spec

## Linked Issue

GH-727 / #727

## Product Spec

Link to `product.md`.

## Current Evidence

At `origin/main@fec54a56`, 2 tracked Rust files remain over the U-16
800-line ceiling. One current largest file is `src/utils/sync/versioned_map.rs`
at 803 lines. It is a production optimistic-locking map utility whose production code ends
before the inline test module; the oversize is caused by unit tests for construction,
insert/get, version lookup, compare-and-swap, retry updates, key/entry listing,
clone-sharing, and concurrent access behavior.

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

## Current Tranche: VersionedMap Test Extraction

### Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| Production sync utility | `src/utils/sync/versioned_map.rs` | Defines `VersionError`, `VersionedEntry<V>`, and `VersionedMap<K, V>` with DashMap/AtomicU64-backed methods. | These public types, methods, and optimistic-locking semantics stay in place. |
| Extracted unit tests | `src/utils/sync/versioned_map_tests.rs` | New path-backed child test module under `versioned_map.rs`. | Keeps tests close to the private module while reducing the production file size. |
| Sync module facade | `src/utils/sync/mod.rs` | Re-exports `ConcurrentVec` and sibling containers. | It remains untouched to keep public import paths stable. |

### Design

1. Keep all production definitions and methods in `src/utils/sync/versioned_map.rs`.
2. Replace the inline `#[cfg(test)] mod tests { ... }` body with:
   - `#[cfg(test)]`
   - `#[path = "versioned_map_tests.rs"]`
   - `mod tests;`
3. Move the original test body into `src/utils/sync/versioned_map_tests.rs` with `use super::*;`.
4. Move tests without assertion, version number, error display, clone-sharing, retry, or thread-concurrency expectation changes.
5. Do not edit `src/utils/sync/mod.rs`, `ConcurrentMap`, `ConcurrentVec`, `AtomicValue`, or unrelated sync containers.

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P1 | `src/utils/sync/versioned_map.rs` | Production `VersionedMap` types and methods remain in the original module with path-backed test delegation. |
| P2 | `src/utils/sync/versioned_map_tests.rs` | Original test names and assertions remain present in the child test module. |
| P3 | VersionedMap behavior | No insert/get/version/CAS/retry/get_or_insert/entries/error-display/clone/concurrent behavior changes. |
| P4 | file size | `wc -l src/utils/sync/versioned_map.rs src/utils/sync/versioned_map_tests.rs` shows both files below 800. |
| P5 | focused test suite | `cargo test utils::sync::versioned_map --lib --all-features` runs the moved tests. |
| P6 | queue count | tracked-file scan shows the remaining queue no longer includes `src/utils/sync/versioned_map.rs`. |

## Risks

- The tests use `std::thread` and `Arc<VersionedMap<_>>`, so the extracted file must retain the original imports and concurrency assertions.
- The focused test filter should use `utils::sync::versioned_map` because the test module remains nested below the sync utility module.
- `VersionedMap` is a shared optimistic-locking utility; this tranche must not modify atomic version strategy, method signatures, retry fallback behavior, or public re-export paths.

## Test Plan

- [ ] `cargo fmt --all -- --check`
- [ ] `git diff --check`
- [ ] `python3 /Users/apple/Desktop/code/AI/tool/specrail/checks/check_workflow.py --repo /Users/apple/Desktop/code/AI/tool/specrail --spec-dir "$PWD/specs/GH727"`
- [ ] `wc -l src/utils/sync/versioned_map.rs src/utils/sync/versioned_map_tests.rs`
- [ ] `cargo test utils::sync::versioned_map --lib --all-features`
- [ ] `cargo check --lib --all-features`
- [ ] `cargo check --all-features --locked`
- [ ] `cargo check`

## Rollback

Move the VersionedMap tests back into `src/utils/sync/versioned_map.rs`
and revert the `specs/GH727` edits. No schema, persistence, or runtime behavior
changes are involved.
