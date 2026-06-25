# GH716 Tasks

Issue: #716
Status: ready for PR

## Tasks

- [x] Search existing docs and issues for workspace split, provider hardening,
  and SpecRail/spec conventions.
- [x] Confirm current repo state and use a clean worktree based on `origin/main`.
- [x] Read #716 and prerequisite architecture issues.
- [x] Draft product spec at `specs/GH716/product.md`.
- [x] Draft tech spec at `specs/GH716/tech.md`.
- [x] Draft RFC at `docs/plan/workspace-crate-split-rfc.md`.
- [x] Run docs and formatting verification.
- [ ] Open PR linked to #716.

## Verification Log

- `test -s specs/GH716/product.md && test -s specs/GH716/tech.md && test -s specs/GH716/tasks.md && test -s docs/plan/workspace-crate-split-rfc.md` passed.
- `rg -n "Proposed Workspace Shape|Feature Compatibility Strategy|Public API And Semver Strategy|Compile-Time And Dependency-Surface Impact|Migration Phases|Required Issue Ordering|#713|#714|#715|#710|#711|#519" docs/plan/workspace-crate-split-rfc.md specs/GH716` passed.
- `cargo fmt --all -- --check` passed.
- `git diff --cached --check` is required after staging because these files are new.
