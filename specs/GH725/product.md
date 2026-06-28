# Product Spec

## Linked Issue

GH-725 / #725

## User Problem

Provider identity is split across enum variants, factory branches, catalog metadata,
and runtime registration. That makes provider additions easy to drift: a selector
can parse, but the factory or runtime registry can disagree about whether it is
constructible or how it is keyed.

## Goals

- Make `PROVIDER_TYPE_REGISTRY` the authoritative source for provider identity
  and dispatchability checks.
- Ensure catalog-backed provider construction follows registry dispatch metadata.
- Ensure runtime `ProviderRegistry` keys providers by canonical provider identity
  unless callers explicitly opt into a logical key.
- Add mechanical tests that catch drift between registry metadata, factory support,
  runtime registration, and selector support.

## Non-Goals

- Do not split provider capability traits; that belongs to GH-729.
- Do not change SDK, `completion()`, or HTTP adapter parity; that belongs to GH-728.
- Do not rewrite the closed `Provider` enum or all dispatch macro arms in this slice.

## Behavior Invariants

1. Every dispatchable non-custom `ProviderType` listed by `PROVIDER_TYPE_REGISTRY`
   is advertised by `Provider::factory_supported_provider_types()`.
2. Every unsupported registry entry is rejected by support-detection helpers and
   falls through to `ProviderError::NotImplemented` during construction.
3. Catalog-backed providers are constructed through entries marked
   `CatalogOpenAiLike`; catalog definitions alone cannot make unsupported enum
   entries appear constructible.
4. `ProviderRegistry::register` stores providers under `Provider::name()`, and
   typed lookup via `get_by_type` reflects the same provider identity.
5. Explicit logical keys remain available through `register_with_key` for multiple
   configured instances of the same provider type.

## Acceptance Criteria

- [ ] Registry dispatchability and factory support are mechanically checked.
- [ ] Catalog provider factory construction uses registry dispatch metadata.
- [ ] Runtime provider registration has tests for canonical key and explicit key behavior.
- [ ] The focused provider registry/factory tests and full feature check pass locally.

## Edge Cases

- Feature-gated providers may be native only when the relevant feature is enabled.
- `OpenAILike` catalog providers report runtime `ProviderType::OpenAICompatible`;
  tests must not treat catalog provider type identity as a native enum identity.
- Duplicate logical provider names remain caller-controlled via `register_with_key`.

## Rollout Notes

This is an internal convergence slice. There is no data migration or public API
change expected, but provider additions should now fail tests earlier when a table
is updated without the corresponding constructibility metadata.
