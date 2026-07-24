# Product Spec

## Linked Issue

GH-1105 / #1105

complexity: medium

## User Problem

Gateway operators cannot configure the runtime router's existing model-alias behavior from YAML. They also cannot assign provider priority, even though priority-based routing reads deployment priorities. This prevents stable public model names and deterministic primary/fallback provider tiers from being represented in deployable configuration.

## Goals

- Allow YAML configuration to map public model aliases to routed model names.
- Allow every provider deployment to inherit an optional provider priority.
- Keep existing configurations and routing behavior unchanged when both fields are omitted.
- Expose configured aliases through the OpenAI model inventory so clients can discover routable names.
- Fail startup explicitly for malformed or cyclic aliases.

## Non-Goals

- Changing provider-specific outbound model mappings.
- Adding new routing strategies or changing weight semantics.
- Adding request-time alias mutation or an alias administration API.
- Defining automatic failover across different model capabilities.

## Behavior Invariants

1. B-001 A non-empty configured model alias resolves to its configured target for normal and streaming routed requests.
2. B-002 Configured aliases are included once in the OpenAI-compatible model inventory while canonical deployment models remain available; startup rejects an alias name that equals any enabled canonical deployment model instead of shadowing that model.
3. B-003 Empty alias names, empty targets, self-aliases, and direct or transitive alias cycles fail configuration/startup explicitly without serving traffic.
4. B-004 Alias declaration order does not affect resolution. Every valid transitive chain is flattened to its final enabled canonical model before publication, including chains longer than the runtime resolver's current 16-hop bound.
5. B-005 A configured provider priority is copied to every deployment created for that provider, and lower numeric priority wins under `priority_based` routing.
6. B-006 Omitted provider priority defaults to `0`. Layered alias maps merge key by key with the later overlay value winning for the same alias; an omitted or empty overlay contributes no keys and therefore preserves base aliases. A standalone omitted or empty map installs no aliases.
7. B-007 Disabled providers create no deployments, so their priority cannot affect routing. Alias shape and graph errors are rejected during configuration validation; canonical-name collisions and final targets without an enabled deployment are rejected after enabled provider models are expanded but before health checks, router publication, or serving traffic.
8. B-008 Unknown YAML fields remain rejected, and alias/priority values are configuration data only: they trigger no network calls, persistence, permissions, or background work.

## Acceptance Criteria

- [ ] YAML parsing and layered-merge tests cover aliases, provider priority, defaults, overlay wins, empty-overlay preservation, and unknown fields.
- [ ] Router construction tests cover direct/transitive aliases, chains longer than 16 hops, cycles, canonical-name collisions, dynamically expanded targets, missing targets, disabled providers, and priority propagation.
- [ ] Route/model inventory tests prove aliases are routable and discoverable.
- [ ] Existing example configuration remains valid without either new field.
- [ ] Configuration documentation contains a working alias plus primary/fallback example.
- [ ] Formatting, all-target/all-feature build, strict Clippy, and full tests pass.

## Edge Cases

- Duplicate YAML mapping keys follow the parser's existing duplicate-key behavior; this feature does not add silent merge rules.
- Layered configuration does not define an alias-deletion sentinel: omitted or explicit `{}` overlays preserve base aliases, while a repeated alias key replaces only that key's target.
- Alias chains may target another alias but must terminate at a model served by an enabled deployment. The installed runtime snapshot maps each alias directly to that final model.
- An alias key may not equal an enabled canonical model name, even when the alias would point back to that same model through another alias.
- Equal provider priorities continue to use the router's existing within-tier selection behavior.
- Very large numeric priorities are valid `u32` values and remain ordered without arithmetic.

## Rollout Notes

The schema additions are optional and default to current behavior. Operators can deploy the new binary first, then add aliases or priorities. Rollback requires removing the new fields before starting an older binary because configuration parsing rejects unknown fields.
