# Consolidated raw audit findings

- Audit date: 2026-05-01
- Target checkout: `main` at `de594c81`
- Raw total: 77 findings: 20 Critical, 27 High, 30 Medium
- Deduplicated execution tracker: 72 items: 20 Critical, 22 High, 30 Medium
- Source: consolidated raw audit list provided in the remediation thread.

## Critical

| ID | Title | File evidence | Impact |
|----|-------|---------------|--------|
| C1 | Anthropic streaming drops `input_json_delta`, `thinking_delta`, `signature_delta`, and `content_block_start` | `src/core/providers/base/sse.rs:499-526,592` | Streaming Claude tool-use unusable; extended-thinking content invisible. |
| C2 | Bedrock Converse silently drops every Tool/Function-role message and ToolResult content parts | `src/core/providers/bedrock/chat/converse.rs:287-313` | Multi-turn tool conversations on Bedrock lose turns and may trigger hallucinated replies. |
| C3 | Anthropic non-stream drops thinking blocks and `cache_creation_input_tokens` / `cache_read_input_tokens` | `src/core/providers/anthropic/client.rs:578-663` | Reasoning content and cache cost savings are invisible. |
| C4 | Public `ChatCompletionDelta` lacks `thinking`, `tool_call_id`, and `refusal` fields | `src/core/streaming/types.rs:81-89`; `src/server/routes/ai/chat.rs:553-560` | Reasoning content is dropped at the gateway boundary. |
| C5 | Gemini tool-call responses emit `finish_reason: Stop` instead of `ToolCalls` | `src/core/providers/gemini/client.rs:469-510` | OpenAI-compatible tool loops on Gemini break silently. |
| C6 | Python LiteLLM-compatible helper hardcodes `tools`, `tool_choice`, and `response_format` to `None` | `src/core/completion/conversion.rs:11-46` | Helper compatibility is broken for tool use and structured output. |
| C7 | MCP transport URLs are validated only by scheme; no SSRF guard | `src/core/mcp/config.rs:128-157`; compare `src/core/a2a/config.rs:231-309` | SSRF to metadata, localhost, and internal admin targets. |
| C8 | pgvector SQL builder interpolates schema/table/threshold and has hand-rolled escaping | `src/core/providers/pg_vector/provider.rs:208-252,443-451`; `src/core/providers/pg_vector/config.rs:317-318` | SQL injection risk if identifiers ever become operator- or tenant-supplied; current validation reduces but does not remove the need for centralized SQL policy. |
| C9 | `debug!` logs full Bedrock/Milvus request bodies outside audit redaction | `src/core/providers/bedrock/client.rs:169-170`; `src/core/providers/milvus/provider.rs:239` | PII and pasted secrets can leak when debug logging is enabled. |
| C10 | `core::streaming::mod.rs` directly imports `actix_web` | `src/core/streaming/mod.rs:7-34` | Core library users drag in HTTP framework dependencies. |
| C11 | Three to four pricing systems coexist | `src/core/cost/`; `src/core/providers/base/pricing.rs`; `src/services/pricing/`; `src/core/providers/anthropic/mod.rs:24-27` | Cost reporting diverges by entry point. |
| C12 | Pricing migration exists but is not registered; `pricing_history` references are phantom | `src/storage/database/migration/m20240201_000001_create_pricing_tables.rs`; `src/storage/database/migration/mod.rs:3-27`; `src/storage/database/entities/pricing.rs` | Fresh DB migrations do not create pricing tables; reactivating entity can break compile. |
| C13 | Budgets are in-memory `DashMap` only | `src/server/state.rs:64`; `src/core/budget/provider_limits.rs:105-107,640-660` | Restart wipes spend tracking. |
| C14 | Rate limiter is per-process `DashMap` with no Redis backing | `src/core/rate_limiter/limiter.rs:5-33` | Multi-replica deployments leak quota and restart resets limits. |
| C15 | Two parallel team/user systems are created and used | `m20240301_000001_create_user_management_tables.rs`; `m20240301_000002_create_teams_table.rs`; SeaORM user/team repositories | Team/user data can split across APIs. |
| C16 | `AuthConfig::default()` returns `enable_jwt: true` with empty `jwt_secret` | `src/config/models/auth.rs:51-63` | Default config is invalid by construction. |
| C17 | Provider trait/dispatch shape blocks easy provider extension | `src/core/traits/provider/llm_provider/trait_definition.rs:42-43`; `src/core/providers/mod.rs:243-313,340-347` | Adding non-OpenAI-format providers requires edits in many places. |
| C18 | Four sources of truth exist for supported providers | `src/core/providers/{mod.rs,provider_type.rs,factory/registry.rs}` | Parseable providers can be unreachable. |
| C19 | Streaming chunk types miss `skip_serializing_if` on optional fields | `src/core/streaming/types.rs:50-89` | OpenAI-compatible clients may misinterpret explicit nulls; bandwidth is inflated. |
| C20 | Cache key uses `DefaultHasher` and omits tool/schema/version dimensions | `src/core/cache/key_generator.rs:30-90,152-170` | Cache can return wrong structured/tool output and is unstable across toolchain changes. |

## High

| ID | Title | File evidence |
|----|-------|---------------|
| H1 | `convert_usage` drops `thinking_usage`; public usage has no matching field | `src/server/routes/ai/chat.rs:479-497` |
| H2 | `parallel_tool_calls`, `extra_body`, `prediction`, `safety_settings`, and `cache_control` are not exposed at HTTP boundary | `src/core/models/openai/requests.rs:14-110`; `src/server/routes/ai/chat.rs:370` |
| H3 | `seed: Option<u32>` cast to `i32` wraps high values | `src/server/routes/ai/chat.rs:373` |
| H4 | Anthropic transform overwrites assistant text when `tool_calls` are present | `src/core/providers/anthropic/client.rs:480-498` |
| H5 | Anthropic Tool-role messages are emitted as plain user text instead of `tool_result` blocks | `src/core/providers/anthropic/client.rs:399-401` |
| H6 | `response_type` field is always `None` on request path | `src/server/routes/ai/chat.rs:334-340` |
| H7 | Catalog guard ordering can shadow explicit Tier-2 branches | `src/core/providers/factory/registry.rs:69,99-175` |
| H8 | `Provider::create_embeddings` and `create_images` only support OpenAI | `src/core/providers/mod.rs:481-512` |
| H9 | Auth rate-limiter `DashMap` is unbounded; cleanup is probabilistic from auth handlers | `src/server/middleware/auth_rate_limiter.rs:8-13,49-59` |
| H10 | `expect("invalid CORS configuration ...")` in app factory can crash hot reload | `src/server/http.rs:113` |
| H11 | JWT validation/example allow weak placeholder-like secrets | `src/config/models/auth.rs:94-117`; `config/gateway.yaml.example:101` |
| H12 | Audit logger silently drops shutdown flush errors and dropped-event sends | `src/core/audit/logger.rs:131-132,166-169` |
| H13 | Many provider directories compile but are unreachable from the five-variant provider enum | `src/core/providers/mod.rs:11-177,340-347`; `src/core/providers/factory/registry.rs:32-180` |
| H14 | SDK reimplements routing/load-balancing separately from `core::router::UnifiedRouter` | `src/sdk/client/routing.rs:11-28` |
| H15 | Several core/provider files exceed the 800 line hard ceiling | `src/core/cost/calculator.rs`; `src/core/providers/anthropic/{models,client}.rs`; `src/core/providers/base/sse.rs` |
| H16 | `src/server/handlers.rs` is an empty placeholder declared in `mod.rs` | `src/server/handlers.rs`; `src/server/mod.rs` |
| H17 | `ProviderType::PydanticAI` is parseable but unreachable and used as unsupported fixture | `src/core/providers/provider_type.rs:36`; `src/core/providers/factory/mod.rs:172-188` |
| H18 | `config/pricing.yaml` exists but is never read | `config/pricing.yaml`; zero `rg "pricing.yaml"` hits in runtime code |
| H19 | Many sites construct `reqwest::Client::new()` with no shared pool/timeouts | raw report list; current local baseline is about 48 production-ish hits |
| H20 | `${ENV_VAR}` substitution leaves missing variables as placeholder text | `src/config/mod.rs:32-57`; `src/config/validation/config_validators.rs:157-159` |
| H21 | Config models lack `#[serde(deny_unknown_fields)]` | repository-wide under `src/config/` |
| H22 | `default_pricing_source()` and example pricing path resolve differently | `src/config/models/gateway.rs:228-242`; `config/gateway.yaml.example:135` |

Five raw High findings are treated as duplicates or sub-cases of the deduplicated tracker rows above, which is why the executable tracker has 22 High rows rather than the raw 27.

## Medium

- M1 Stream logprobs serialization swallows error via `.ok()` (`chat.rs:562-564`).
- M2 `ChatCompletionRequest` has no `#[serde(flatten)]` extra catcher (`requests.rs:14-75`).
- M3 LiteLLM helper drops thinking/reasoning/metadata (`completion/conversion.rs:40-44`).
- M4 Bedrock Image/Audio/Document content parts are dropped silently (`bedrock/chat/converse.rs:263-298`).
- M5 Anthropic `stop_sequence`, `refusal`, and `pause_turn` collapse to `Stop` (`anthropic/client.rs:633-637`).
- M6 Gemini `cachedContentTokenCount` and `thoughtsTokenCount` are dropped (`gemini/client.rs:514-530`).
- M7 A2A SSRF guard is host-string based and has DNS rebinding risk (`a2a/config.rs:250-309`).
- M8 MCP tool descriptions are cached without a hash baseline (`mcp/server.rs:49-50,187-217`).
- M9 Audit redactor misses Bearer JWTs, gateway keys, AWS keys, and Anthropic keys (`audit/config.rs:111-116`).
- M10 Forgot-password path has timing-based account enumeration risk (`auth/password.rs:50-56`).
- M11 `FileSecretManager::read_secret` checks `path.exists()` before validation (`secret_managers/file.rs:117-125`).
- M12 Upstream provider error body is forwarded verbatim to the user (`openai_like/provider.rs:406`).
- M13 `LITELLM_SQLITE_PATH` and `LITELLM_DATA_DIR` split sibling local state.
- M14 `AppState.config: AtomicValue<Config>` claims hot reload but no watcher exists (`server/state.rs:21-26`).
- M15 `redis.enabled: true` in example does not affect several consumers.
- M16 Multiple `ModelPricing` structs exist; one in `mod.rs:218-226` is only used by self-tests.
- M17 `Provider::calculate_cost` ignores provider name (`mod.rs:449-463`).
- M18 `OpenAILikeProvider` leaks name strings via `Box::leak`.
- M19 `mcp::AuthType` collides with auth namespace concepts.
- M20 `lite` feature is not reachable from default dependency graph and needs CI coverage.
- M21 `analytics -> metrics -> sysinfo` couples unrelated concerns (`Cargo.toml:188-197`).
- M22 `vector-db = []` is an empty feature flag (`Cargo.toml:195`).
- M23 Six provider macro modules are likely obsolete after catalog convergence.
- M24 `paste` is aliased as `pastey`; remove the rename.
- M25 `lib.rs` exposes a broad root re-export surface.
- M26 `default_pricing_source()` has cwd-vs-exe relative inconsistency.
- M27 Two team-table tracks are created in every fresh DB.
- M28 `core::streaming::providers` is a parallel SSE pipeline beside `base/sse.rs`.
- M29 Cache key cardinality/prefixing is weak around 64-bit `DefaultHasher`.
- M30 Outbound `reqwest::Client` constructions also lack shared proxy/user-agent config.

## Cross-agent confirmations

1. Cache key correctness: Agents 1 and 4.
2. Per-request `reqwest::Client::new()`: Agents 2 and 4.
3. JWT secret weak validation: Agents 2 and 4.
4. MCP SSRF gap: Agents 2 and 4.
5. In-memory budget/rate-limit state: Agents 2 and 4.
6. Anthropic streaming/content gaps: Agents 1 and 3.

## Method limits

- Architecture agent could not run all verification commands; some file-size and orphan-module counts are confidence-qualified.
- Prior memory referenced a `crates/*` and `apps/*` workspace split that does not exist in this checkout.
- The named `litellm-provider-core::DynProvider` from memory also does not exist in this checkout.
- Some medium-confidence raw findings require local `#[cfg(test)]` placement checks before editing.
