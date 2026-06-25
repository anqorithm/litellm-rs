# GH716 Tech Spec: Workspace Crate Split RFC

Product spec: `specs/GH716/product.md`
RFC: `docs/plan/workspace-crate-split-rfc.md`
Issue: #716

## Context

- `Cargo.toml` defines a single package named `litellm-rs` and a single library
  crate named `litellm_rs`.
- `Cargo.toml` defaults to `sqlite`, `redis`, `metrics`, and `tracing`.
- `Cargo.toml` makes `sqlite` enable `storage`, and `storage` enable
  `gateway` plus `redis`.
- `src/lib.rs` exports SDK, core, provider, router, gateway, server, and storage
  surfaces from the same crate.
- `src/core/providers/provider_type.rs` contains the closed `ProviderType` enum
  that is central to provider wiring.
- `src/core/providers/unified_provider.rs` documents retry and HTTP mapping as
  responsibilities of the provider error type.
- `src/core/budget/tracker.rs` exposes separate `check_spend` and
  `record_spend`, which is still tracked by #711.
- `docs/plan/router-budget-provider-infra-hardening-spec.md` already places
  #716 after router, budget, provider, registry, and retry/error hardening.

## Proposed Changes

1. Add `docs/plan/workspace-crate-split-rfc.md`.
2. Add `specs/GH716/product.md`, `specs/GH716/tech.md`, and
   `specs/GH716/tasks.md`.
3. Keep the RFC docs-only and explicitly block workspace implementation until
   prerequisite architecture issues are accepted.
4. Use the existing `docs/plan/` convention for the RFC because this repository
   already keeps architecture execution plans there.

## Workspace Boundary Proposal

The RFC proposes these target crates:

- `litellm-core`: canonical shared types and low-level primitives.
- `litellm-provider-api`: provider traits, capabilities, metadata, and
  fact-only failure contracts.
- provider implementation crates: grouped provider implementations first,
  separate heavy provider crates only when justified.
- `litellm-router`: deployment selection, routing strategies, health/cooldown,
  reservation, and retry integration.
- `litellm-gateway`: Actix HTTP server, auth, adapters, CLI, and gateway
  binary.
- `litellm-storage`: SeaORM, Redis, S3/object storage, migrations, and
  persistence repositories.
- `litellm-rs`: compatibility facade preserving current imports and feature
  names during migration.

## Prerequisite Issues

Implementation of the split should wait on accepted direction for:

- #713: provider enum, trait, and handle contract alignment.
- #714: provider declaration source and conformance tests.
- #715: provider failure facts versus retry policy and HTTP mapping.
- #710: atomic router metadata snapshot boundary.
- #711: budget reserve and settle semantics.
- #519: duplicate type tree and provider abstraction roadmap.

The RFC can be reviewed while these are open. The code split should not start
until the accepted contracts are clear.

## Testing And Validation

| Product Behavior | Verification |
| --- | --- |
| RFC exists and covers #716 acceptance topics | `rg -n "proposed workspace|feature compatibility|semver|dependency-surface|migration phases|Required Issue Ordering" docs/plan/workspace-crate-split-rfc.md` |
| Spec packet exists | `test -s specs/GH716/product.md && test -s specs/GH716/tech.md && test -s specs/GH716/tasks.md` |
| Docs-only change has no whitespace errors | `git diff --check` |
| Rust formatting is unaffected | `cargo fmt --all -- --check` |

Full `cargo test` is not required for this docs-only RFC PR because no Rust
source, Cargo manifest, migrations, or test files change.

## Risks

- The RFC may be accepted before prerequisite contracts are implemented; the RFC
  mitigates this by separating "review RFC" from "start split".
- Provider crate granularity can be over-designed; the RFC mitigates this by
  starting with grouped provider crates.
- The compatibility facade can hide dependency regressions; the RFC requires
  `cargo tree -e features` baselines before implementation.

## Follow-Ups

- After review, create implementation issues for Phase 0 and Phase 1 only.
- Add compile-time and dependency-tree baselines before the first code split PR.
- Decide whether an eventual `litellm-sdk` crate is needed.
