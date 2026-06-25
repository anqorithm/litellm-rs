# GH716 Product Spec: Workspace Crate Split RFC

Issue: #716
Status: Draft
Locale: en-US

## Summary

Produce a reviewable RFC for splitting `litellm-rs` into smaller workspace
crates without starting the implementation split. The RFC should let maintainers
decide the migration direction, dependency-surface goals, and prerequisite
architecture work before public crate boundaries are introduced.

## Users

- SDK-only Rust users who want LiteLLM-compatible APIs without gateway,
  storage, Redis, or auth dependencies.
- Gateway operators who want the existing default gateway experience to remain
  convenient.
- Provider contributors who need clear provider API and registry boundaries.
- Maintainers who need smaller semver surfaces and safer staged releases.

## Behavior

1. A maintainer can review one RFC document that explains the target workspace
   crate layout, ownership boundaries, compatibility strategy, migration phases,
   and prerequisite issues.
2. The RFC keeps the current `litellm-rs` package as a compatibility facade
   during migration.
3. The RFC preserves the current gateway-oriented default behavior in the first
   migration phase and calls out any future feature graph change as a separate
   compatibility decision.
4. The RFC explains how SDK-only usage should become lighter after the split.
5. The RFC does not change code behavior, Cargo features, public APIs, or CI
   behavior in this PR.
6. The spec packet records the product and technical intent under
   `specs/GH716/`.

## Non-Goals

- Do not move Rust modules into new crates.
- Do not edit `Cargo.toml` feature behavior.
- Do not close prerequisite architecture issues by documentation alone.
- Do not claim compile-time improvement without a later implementation PR and
  fresh measurement.

## Acceptance Criteria

- `docs/plan/workspace-crate-split-rfc.md` exists and is linked to #716.
- The RFC covers proposed workspace crates, feature compatibility, public API
  and semver strategy, compile-time and dependency-surface impact, migration
  phases, and required predecessor issues.
- `specs/GH716/tech.md` maps the RFC to current repository evidence and
  validation commands.
- `specs/GH716/tasks.md` records the completed documentation tasks.
- No Rust source files are modified.

## Open Questions

- Should the eventual SDK surface become a separate `litellm-sdk` crate, or
  stay as facade exports over `litellm-core` and provider API crates?
- How long should the current gateway-oriented default feature set remain the
  facade default after the split starts?
