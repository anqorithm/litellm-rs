# Tech Spec

## Linked Issue

GH-725 / #725

## Product Spec

Link to `product.md`.

## Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| Provider metadata | `src/core/providers/registry/types.rs` | `PROVIDER_TYPE_REGISTRY` records canonical names, aliases, dispatch kind, and catalog backing. | This is the intended authoritative matrix. |
| Factory dispatch | `src/core/providers/factory/registry.rs` | Native and explicit OpenAI-like branches are handwritten; catalog fallback checks catalog definitions directly. | Catalog constructibility should be gated by registry dispatch metadata. |
| Runtime registry | `src/core/providers/provider_registry.rs` | `register()` uses `provider.name()` but only empty-registry behavior is tested. | Runtime keying should be guarded against identity drift. |
| Selector support | `src/core/providers/factory/resolver.rs` | Catalog definitions and factory-supported types decide support. | Unsupported registry entries must not be accepted accidentally. |
| Default router boot | `src/core/completion/default_router/mod.rs` | Several catalog provider registrations are manually repeated. | Runtime catalog registration can be driven by the catalog/registry path. |

## Proposed Design

Keep the closed `Provider` enum intact for this issue. The high-risk enum and
macro-generation refactor is deferred.

Tighten the lower-risk convergence path:

- Add a registry helper that returns only catalog-dispatch entries.
- Use that helper in `Provider::from_config_async` so catalog fallback depends on
  `ProviderDispatchKind::CatalogOpenAiLike`, not on catalog presence alone.
- Use catalog iteration in `DefaultRouter::new` to register known environment-backed
  OpenAI-like providers once, removing repeated hand-written blocks for the same
  provider names.
- Extend tests so registry dispatchability, selector support, factory construction,
  and runtime registration are checked together.

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P1 | `registry/types.rs`, `providers/mod.rs` | `cargo test provider_registry --lib` |
| P2 | `factory/resolver.rs`, `factory/registry.rs` | `cargo test from_config_async --lib` |
| P3 | `factory/registry.rs` | `cargo test dispatch_kind_matches_runtime_variant --lib` |
| P4 | `provider_registry.rs` | `cargo test provider_registry --lib` |
| P5 | `provider_registry.rs` | `cargo test register_with_key --lib` |

## Data Flow

Input provider selectors parse into `ProviderType` or catalog names. Registry
entries define whether that selector is constructible natively, explicitly as an
OpenAI-like provider, via the catalog, or not at all. Factory construction returns
a `Provider` enum instance, and runtime registries store that instance by canonical
provider name unless an explicit logical key is supplied.

No persistence or external calls are added.

## Alternatives Considered

- Generate the full `Provider` enum and four dispatch macro arms from one macro.
  Deferred because feature-gated provider modules make this a broader refactor.
- Remove the enum and use trait objects in runtime registries. Deferred because it
  changes router dispatch semantics and is larger than GH-725.

## Risks

- Security: no secret handling changes; tests must avoid real API keys.
- Compatibility: provider selector support should not expand or shrink except where
  it already follows dispatch metadata.
- Performance: registry iteration happens at startup/tests only and uses static data.
- Maintenance: tests should describe the remaining enum/manual-match boundary rather
  than overclaiming full generation.

## Test Plan

- [ ] Unit tests: `cargo test provider_registry --lib`
- [ ] Unit tests: `cargo test from_config_async --lib`
- [ ] Static check: `cargo fmt --all -- --check`
- [ ] Full feature compile: `cargo check --all-features --locked`

## Rollback Plan

Revert the GH-725 commit or PR. The changes are limited to provider registry,
factory dispatch, router startup catalog iteration, tests, and the SpecRail packet.
