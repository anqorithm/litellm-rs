# Tech Spec

## Linked Issue

GH-727 / #727

## Product Spec

Link to `product.md`.

## Current Evidence

At `origin/main@1bbe63fd`, the full tracked Rust scan reports one remaining
file over the U-16 800-line ceiling:

- `803 src/core/user_management/types.rs`

`types.rs` defines four public entity structs (`User`, `Team`, `Organization`,
`TeamMember`) and then carries an inline test module covering construction,
minimal variants, clone/debug/serde, role variants, relationship checks, budget
simulation, active-member filtering, and deserialization. The production type
surface itself is small; the oversize is caused by inline tests.

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
7. Closure honesty: closing keywords are allowed only in the final tranche after a clean
   over-800 scan shows no remaining tracked Rust files.

## Queue Design

| Phase | Lane | Target examples | Verification pattern |
| --- | --- | --- | --- |
| P1 | Test suites | DataUtils tests, router tests, utils/event tests, provider test files, integration route tests | focused `cargo test` for the moved module plus line-count proof |
| P2 | Public type facades | SDK, analytics, monitoring, observability, audio, config, user/key/cache types | choose test extraction when tests cause the oversize; otherwise use facade + `pub use` |
| P3 | Runtime orchestrators | OpenTelemetry, OAuth session, request validator | focused module tests or affected integration tests plus all-features check |
| P4 | Utility modules | config helpers, net client utils, sync containers | focused utility tests plus concurrency/behavior checks where relevant |
| P5 | Closure scan | all Rust files | full over-800 scan, final SpecRail update, final PR may close #727 |

## Current Tranche: User-Management Type Test Extraction

### Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| Production user-management DTOs | `src/core/user_management/types.rs` | Defines `User`, `Team`, `Organization`, and `TeamMember` with serde derives and field-level contracts. | These public types and fields stay in place. |
| Extracted unit tests | `src/core/user_management/types_tests.rs` | New path-backed child test module under `types.rs`. | Keeps tests close to the private module while reducing production file size. |
| Module facade | `src/core/user_management/mod.rs` | Re-exports `Organization`, `Team`, `TeamMember`, and `User` from `types`. | It remains untouched to keep public import paths stable. |

### Design

1. Keep all production entity definitions in `src/core/user_management/types.rs`.
2. Replace the inline `#[cfg(test)] mod tests { ... }` body with:
   - `#[cfg(test)]`
   - `#[path = "types_tests.rs"]`
   - `mod tests;`
3. Move the original test body into `src/core/user_management/types_tests.rs` with `use super::*;`.
4. Move tests without assertion, fixture, role-list, relationship, budget, active-member, or JSON deserialization changes.
5. Do not edit `src/core/user_management/mod.rs`, `roles.rs`, `settings.rs`, `manager.rs`, `team_ops.rs`, or `user_ops.rs`.
6. Run a final tracked Rust scan after the split; it must return no files.

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P1 | `src/core/user_management/types.rs` | Production user-management entity structs remain in the original module with path-backed test delegation. |
| P2 | `src/core/user_management/types_tests.rs` | Original test names and assertions remain present in the child test module. |
| P3 | Public API compatibility | `src/core/user_management/mod.rs` is untouched and still re-exports the four entity types. |
| P4 | serde/role/relationship behavior | Focused `cargo test core::user_management::types --lib --all-features` runs the moved tests. |
| P5 | file size | `wc -l src/core/user_management/types.rs src/core/user_management/types_tests.rs` shows both files below 800. |
| P6 | closure scan | tracked-file over-800 scan returns no files. |

## Risks

- The test module depends on parent imports for `HashMap`, `Utc`, roles, and settings defaults; the extracted file must retain `use super::*;`.
- The focused test filter should use `core::user_management::types` because the test module remains nested below `types.rs`.
- This is the final #727 tranche, so the PR body must not claim closure until the local scan is empty and PR gates pass.

## Test Plan

- [ ] `cargo fmt --all -- --check`
- [ ] `git diff --check`
- [ ] `python3 /Users/apple/Desktop/code/AI/tool/specrail/checks/check_workflow.py --repo /Users/apple/Desktop/code/AI/tool/specrail --spec-dir "$PWD/specs/GH727"`
- [ ] `wc -l src/core/user_management/types.rs src/core/user_management/types_tests.rs`
- [ ] `cargo test core::user_management::types --lib --all-features`
- [ ] `rg --files -g '*.rs' src tests | xargs wc -l | awk '$1 > 800 && $2 != "total" { print $1 " " $2 }' | sort -nr`
- [ ] `cargo check --lib --all-features`
- [ ] `cargo check --all-features --locked`
- [ ] `cargo check`

## Rollback

Move the user-management type tests back into `src/core/user_management/types.rs`
and revert the `specs/GH727` edits. No schema, persistence, public API, or runtime
behavior changes are involved.
