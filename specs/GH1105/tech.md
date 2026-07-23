# Tech Spec

## Linked Issue

GH-1105 / #1105

## Product Spec

See `product.md`.

## Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| Gateway schema | `src/config/models/gateway.rs`, `src/config/models/provider.rs` | Gateway has no alias map; providers have weight but no priority | B-003/B-006/B-008 schema and defaults |
| Config validation | `src/config/validation/config_validators.rs` | Validates providers and router independently | Alias shape and target validation must fail before startup |
| Router construction | `src/core/router/gateway_config.rs` | Creates deployments, hardcodes priority `0`, installs no aliases | B-001/B-004/B-005/B-007 root behavior |
| Server startup | `src/server/http.rs` | Builds the unified router from providers and runtime router settings | Must pass validated aliases into construction atomically |
| Model route | `src/server/routes/ai/models.rs` | Lists deployment model names only | B-002 discoverability |
| Examples/docs | `config/gateway.yaml.example`, `README.md` | Documents providers and router fields | Migration and operator contract |

## Proposed Design

1. Add `model_aliases: HashMap<String, String>` to `GatewayConfig` with `#[serde(default)]`, include it in defaults/export behavior, and add `priority: u32` to `ProviderConfig` with `#[serde(default)]` and debug/default handling. `GatewayConfig::merge` starts with the base alias map and inserts every overlay entry, so overlay values win per key. Because serde maps omitted and explicit `{}` to the same empty map, either form preserves all base entries; this version defines no clearing sentinel. Add focused tests for disjoint union, same-key override, omitted overlay, and explicit-empty overlay.
2. Split validation into two deterministic phases. Phase A, in configuration validation, rejects trimmed-empty aliases/targets, self references, and direct or transitive cycles using per-alias traversal independent of `HashMap` iteration order. It does not decide target availability. Phase B, during router construction, first creates enabled provider instances and expands each deployment's canonical models from configured `models`, otherwise `provider.list_models()`, otherwise the existing provider-name fallback. From that staged deployment set, reject alias keys colliding with any canonical model and reject chains whose final target is absent.
3. Add a backward-compatible router constructor that accepts aliases, or extend construction through a private helper while preserving the existing public `from_gateway_config` API. Propagate `ProviderConfig.priority` into every staged `DeploymentConfig`. Resolve every Phase-A-valid alias chain against the Phase-B canonical model set and build a normalized map in which every alias points directly to its final canonical model. Install the staged deployments and complete normalized alias map as one unpublished construction result before starting health checks; no partially validated router reaches `AppState`.
4. Expose a read-only router alias snapshot/accessor and merge alias names into `/v1/models`, sorted and deduplicated using the route's existing deterministic output behavior.
5. Document direct aliases, transitive aliases, priority ordering, default `0`, and rollback compatibility in the example config and README.

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| B-001 | gateway config plus router construction | direct alias completion-selection and streaming-selection focused tests |
| B-002 | Phase-B collision validation, router alias accessor, and model route | canonical-key collision fails; otherwise inventory contains alias and canonical model exactly once |
| B-003 | gateway validation and runtime alias installation | empty/self/direct-cycle/transitive-cycle negative tests |
| B-004 | deterministic graph validation, chain flattening, and batch installation | reversed-order and greater-than-16-hop chains install identical single-hop mappings |
| B-005 | provider config to deployment config | configured tiers select lower priority first |
| B-006 | serde/defaults, key-wise merge, and legacy constructor | disjoint/same-key/omitted/explicit-empty overlay tests plus old YAML with priority-zero deployments and no aliases |
| B-007 | two-phase validation against expanded enabled-provider models | configured and dynamic model targets succeed; canonical collision and disabled-only/missing targets fail before health checks or server state creation |
| B-008 | deny-unknown schema and pure validation | unknown-field regression and no-I/O unit coverage |

## Data Flow

YAML is environment-substituted and deserialized into `GatewayConfig`; layered configs merge alias entries key by key. Phase A validates alias shape and graph structure without assuming that YAML declares every model. Server startup then instantiates enabled providers, expands each provider's effective models, and stages the corresponding deployments while copying priority. Phase B derives the canonical model set from those staged deployments, rejects alias-key collisions and missing final targets, and flattens each valid alias to that final canonical model. The complete deployments-plus-aliases state is installed before health checks and `AppState` publication. Routed requests therefore need at most one alias lookup, and the model route reads canonical models and alias names from that same router state.

## Alternatives Considered

- Put aliases in each provider: rejected because inbound public names belong to routing, while provider model mappings translate outbound names.
- Put aliases in runtime-only `RouterConfig`: rejected because that type is also used by SDK/runtime callers and should not acquire deployment inventory policy.
- Install aliases after `AppState` creation: rejected because a partially initialized router could become observable if alias installation fails.
- Treat missing targets as valid future aliases: rejected because a typo would make startup succeed while every request fails at runtime.
- Validate final targets only from YAML `models`: rejected because an empty list is expanded from `provider.list_models()` during construction and would falsely reject valid dynamic models.
- Keep transitive chains in the runtime map: rejected because the current resolver stops after 16 hops and could silently return an intermediate alias.

## Risks

- Security: aliases and priorities are local configuration only; validation must not resolve URLs or emit secrets.
- Compatibility: older binaries reject the new fields, so rollback requires reverting config first; omitted fields preserve current behavior.
- Performance: startup validation is linear in aliases times maximum chain length; flattening makes configured-alias resolution a single map hop at request time.
- Maintenance: model inventory must use the same router alias state as routing to avoid config/runtime drift.

## Test Plan

- [ ] Unit tests: serde/default/debug, key-wise overlay merge, Phase-A graph validation, priority propagation, normalized alias accessor.
- [ ] Integration tests: gateway construction with configured and dynamically expanded models, Phase-B collision/target failures, a greater-than-16-hop chain, request selection, and model inventory using test providers without external calls.
- [ ] Manual verification: start with example YAML, query `/v1/models`, and send a completion through an alias.
- [ ] Repository gates: format, all-target/all-feature check, strict Clippy, and serial full test.

## Rollback Plan

Remove `model_aliases` and `priority` from deployed YAML, verify the old configuration parses, then roll back the binary. No data migration or persistent state rollback is required.
