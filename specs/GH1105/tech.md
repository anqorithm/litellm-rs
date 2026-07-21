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

1. Add `model_aliases: HashMap<String, String>` to `GatewayConfig` with `#[serde(default)]`, include it in defaults/merge/export behavior, and add `priority: u32` to `ProviderConfig` with `#[serde(default)]` and debug/default handling.
2. Extend gateway validation to reject trimmed-empty aliases/targets, self references, cycles, and chains whose final canonical target is not served by an enabled provider. Use map traversal with a per-alias visited set; do not depend on hash iteration order.
3. Add a backward-compatible router constructor that accepts aliases, or extend construction through a private helper while preserving the existing public `from_gateway_config` API. Build enabled deployments first, propagate `ProviderConfig.priority` into `DeploymentConfig`, then install all validated aliases before health checks start.
4. Expose a read-only router alias snapshot/accessor and merge alias names into `/v1/models`, sorted and deduplicated using the route's existing deterministic output behavior.
5. Document direct aliases, transitive aliases, priority ordering, default `0`, and rollback compatibility in the example config and README.

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| B-001 | gateway config plus router construction | direct alias completion-selection and streaming-selection focused tests |
| B-002 | router alias accessor plus model route | inventory contains alias and canonical model exactly once |
| B-003 | gateway validation and runtime alias installation | empty/self/direct-cycle/transitive-cycle negative tests |
| B-004 | deterministic graph validation/installation | reversed-order transitive alias test resolves identically |
| B-005 | provider config to deployment config | configured tiers select lower priority first |
| B-006 | serde/defaults and legacy constructor | old YAML parses and creates priority-zero deployments with no aliases |
| B-007 | validation against enabled-provider models | disabled-only/missing canonical target fails before server state creation |
| B-008 | deny-unknown schema and pure validation | unknown-field regression and no-I/O unit coverage |

## Data Flow

YAML is environment-substituted and deserialized into `GatewayConfig`. Validation checks the alias graph against models declared by enabled providers. Server startup converts provider entries into runtime deployments, copying priority, then installs aliases into the router snapshot before health checks and `AppState` publication. Routed requests resolve aliases through the existing snapshot path. The model route reads deployment models and configured alias names from the same router state.

## Alternatives Considered

- Put aliases in each provider: rejected because inbound public names belong to routing, while provider model mappings translate outbound names.
- Put aliases in runtime-only `RouterConfig`: rejected because that type is also used by SDK/runtime callers and should not acquire deployment inventory policy.
- Install aliases after `AppState` creation: rejected because a partially initialized router could become observable if alias installation fails.
- Treat missing targets as valid future aliases: rejected because a typo would make startup succeed while every request fails at runtime.

## Risks

- Security: aliases and priorities are local configuration only; validation must not resolve URLs or emit secrets.
- Compatibility: older binaries reject the new fields, so rollback requires reverting config first; omitted fields preserve current behavior.
- Performance: startup validation is linear in aliases times maximum chain length; request resolution remains the existing router snapshot lookup.
- Maintenance: model inventory must use the same router alias state as routing to avoid config/runtime drift.

## Test Plan

- [ ] Unit tests: serde/default/debug/merge, graph validation, priority propagation, alias accessor.
- [ ] Integration tests: gateway construction, request selection, and model inventory using test providers without external calls.
- [ ] Manual verification: start with example YAML, query `/v1/models`, and send a completion through an alias.
- [ ] Repository gates: format, all-target/all-feature check, strict Clippy, and serial full test.

## Rollback Plan

Remove `model_aliases` and `priority` from deployed YAML, verify the old configuration parses, then roll back the binary. No data migration or persistent state rollback is required.
