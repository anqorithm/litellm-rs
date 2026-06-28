# Tech Spec

## Linked Issue

GH-729 / #729

## Product Spec

Link to `product.md`.

## Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| Provider trait | `src/core/traits/provider/llm_provider/trait_definition.rs` | `LLMProvider` exposes `capabilities()` plus per-capability helper methods. Optional methods return `NotSupported` by default. | This is the canonical provider behavior API. |
| Legacy sub-traits | `src/core/traits/provider/llm_provider/sub_traits.rs` | Deprecated blanket adapters over `LLMProvider`, but wording still says future roadmap may finish the carve-out. | This is the unused architecture surface called out by #729. |
| Provider enum | `src/core/providers/mod.rs` | Router deployments store `Provider`; optional calls dispatch to `LLMProvider` methods through enum arms. | This is the current runtime dispatch boundary. |
| Router capability selection | `src/core/router/selection.rs`, `src/core/router/unified.rs`, `src/server/routes/ai/provider_selection.rs` | These paths scan `provider.capabilities()` directly. | These should share one capability predicate. |

## 设计方案

Use `ProviderCapability` as the enforced runtime capability model for this issue.

1. Add `LLMProvider::supports_capability(&ProviderCapability)` as the canonical predicate over `capabilities()`.
2. Rewrite `supports_streaming()`, `supports_embeddings()`, and `supports_image_generation()` to delegate to `supports_capability()`.
3. Add `Provider::supports_capability(&ProviderCapability)` and use it in router/server capability scans.
4. Update `sub_traits.rs` top-level guidance and deprecation notes to say these traits are legacy compatibility adapters, not routing contracts.
5. Add tests with a local chat-only mock and optional-capability mock so the contract is independent of concrete provider catalog drift.

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P1 | `LLMProvider::supports_capability` | Unit test checks chat-only and embeddings-capable mocks. |
| P2 | helper methods delegate to canonical predicate | Unit test asserts stream/embed/image helpers mirror capability list. |
| P3 | optional method failure remains explicit | Existing provider enum tests plus focused sub-trait/trait tests. |
| P4 | sub-traits remain compatibility-only | Documentation update and deprecated adapter tests continue passing. |
| P5 | router/gateway uses one predicate | Compile check plus focused router/provider tests. |

## 数据流

Provider implementations return a static `&[ProviderCapability]`. Runtime router selection asks the selected `Provider` enum whether it supports a requested capability. Optional execution methods are called only after routing selects a capable deployment; if called directly on an unsupported provider, the `LLMProvider` default returns `ProviderError::NotSupported`.

No persistence, config format, or external API payload changes are required.

## 备选方案

- Real sub-trait carve-out: rejected for this issue because router deployments still use the closed `Provider` enum and a full carve-out would overlap factory/registry work.
- Delete `sub_traits.rs`: rejected for compatibility; this can wait for a future major release.

## 风险

- Security: No auth, secret, or request execution changes.
- Compatibility: Adding default trait methods is source-compatible for existing provider implementations; deleting symbols is avoided.
- Performance: Capability checks remain a small slice scan over static arrays.
- Maintenance: This makes future provider work less ambiguous by naming one predicate.

## 测试计划

- [ ] Unit tests: focused trait/provider/router capability tests.
- [ ] Integration tests: existing router capability selection tests.
- [ ] Manual verification: inspect diff to confirm no factory/registry/provider implementation churn.
- [ ] Full check: `cargo check --all-features --locked`.

## 回滚方案

Revert the trait helper, provider helper, router predicate call-site changes, documentation updates, and `specs/GH729` packet. Because no data migration or config change is introduced, rollback is a normal code revert.
