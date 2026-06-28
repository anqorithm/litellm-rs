# Tech Spec

## Linked Issue

GH-728 / #728

## Product Spec

Link to `product.md`.

## Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| Provider registry | `src/core/providers/registry/*` | Registry knows provider identity, aliases, catalog entries, and factory dispatch. | Best home for a cross-surface support matrix. |
| SDK routing | `src/sdk/client/routing.rs` | Local `supports_chat` marks Google chat-capable even though execution is not implemented. | Main bug called out by #728. |
| SDK execution | `src/sdk/client/completions.rs`, `src/sdk/client/embeddings.rs` | Execution has provider-specific branches and mixed unsupported errors. | Direct calls must match routing support state. |
| completion() router | `src/core/completion/default_router/*` | Dynamic/static prefixes are separate from provider registry support documentation. | Needs explicit unsupported behavior for known unsupported prefixes. |
| Docs | `README.md` | Provider matrix covers provider runtime capabilities but not SDK/completion surfaces. | Acceptance requires one documented matrix across public surfaces. |

## 设计方案

1. Add `src/core/providers/registry/support_matrix.rs`.
   - Define `ProviderRouteSurface` for HTTP chat/stream/embeddings/images, SDK chat/stream/embeddings, and `completion()` chat/stream.
   - Define `SurfaceSupport` with `Supported`, `Passthrough`, `FeatureGated`, and `Unsupported`.
   - Provide `supports_provider_surface(provider, surface)` and canonical selector normalization.
2. Export support matrix helpers from `registry/mod.rs`.
3. Replace SDK routing's local chat/stream provider lists with matrix-backed checks.
   - Default provider path also checks matrix support.
   - If a model matches only unsupported providers, return `SDKError::NotSupported` instead of `ModelNotFound`.
4. Guard SDK direct execution with the same matrix helpers.
   - Google chat returns `NotSupported`.
   - Embeddings still use existing prefix/base_url validation after matrix support passes.
5. Add a minimal `completion()` prefix guard for explicit matrix rows that are unsupported.
   - `google/...` and `gemini/...` become explicit bad requests.
   - Unlisted catalog-like prefixes remain available to generic `api_base` passthrough.
6. Update README with a route-surface matrix.

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P1 | `support_matrix.rs` | Unit tests cover registry/catalog selectors and default completion routes. |
| P2 | SDK routing | Tests reject Google SDK chat and unsupported default provider. |
| P3 | SDK direct execution | Test asserts `execute_chat_request("google", ...)` returns `NotSupported`. |
| P4 | Embeddings support | Existing embeddings tests continue to pass; matrix adds a pre-check only. |
| P5 | completion() explicit unsupported | Unit test checks Google/Gemini prefixes and does not block unlisted catalog prefixes. |
| P6 | Docs | README updated and registry README tests remain valid. |

## 数据流

Callers pass a provider selector and a `ProviderRouteSurface` into the support
matrix. The selector is normalized through registry aliases where possible.
Explicit rows win; otherwise Tier 1 catalog providers get a conservative HTTP
chat/stream fallback. SDK code maps its `ProviderType` enum to these canonical
selectors before routing or execution.

## 备选方案

- Generate the matrix from every concrete provider's runtime `capabilities()`:
  rejected for this slice because SDK and `completion()` support are adapter
  availability questions, not only core provider capability questions.
- Implement Google SDK chat now: rejected because it requires request/response
  payload work and auth behavior outside #728's matrix contract.
- Document only README state: rejected because SDK routing would continue to
  drift.

## 风险

- Compatibility: Google SDK chat changes error class from `ProviderError` to
  `NotSupported`; this is intentional and more stable.
- Feature flags: feature-gated HTTP rows use `cfg!(feature = ...)` so SDK
  selection does not accidentally choose unavailable adapters.
- False negatives: explicit `completion()` prefix guard is limited to matrix
  rows, so generic custom `api_base` routes for unlisted names remain possible.
- Security: no auth, secret, or request signing changes.

## 测试计划

- [ ] SpecRail packet validation.
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo test support_matrix --lib`
- [ ] `cargo test sdk_chat_routing --lib`
- [ ] `cargo test google_returns_not_supported --lib`
- [ ] `cargo test unsupported_completion_prefixes_are_explicit --lib`
- [ ] `cargo test sdk_provider_support --lib`
- [ ] `cargo check --all-features --locked`

## 回滚方案

Revert the support matrix module, SDK matrix wiring, completion prefix guard,
README matrix, and `specs/GH728`. No data migration or config rewrite is
introduced.
