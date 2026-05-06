# Audit Remediation Spec — litellm-rs

> **ARCHIVED — 2026-05-06.** This is a post-hoc parallel spec written from chat-side analysis. The canonical execution record is [`docs/plan/audit-remediation-complete-plan.md`](../../plan/audit-remediation-complete-plan.md), and the closure summary is [`closeout-2026-05-02.md`](../closeout-2026-05-02.md). The 41 remediation steps closed via PR #463–#495 on 2026-05-02. This file is preserved only for cross-reference; do not treat it as a live tracker.

**Audit date**: 2026-05-01
**Authors**: codebase-audit (4 parallel opus agents — API/Data, Security, Architecture, Config/Persistence)
**Target commit**: `de594c81` (branch `main`)
**Status legend**: `TODO` · `IN_PROGRESS` · `BLOCKED` · `DONE`
**Confidence labels** (per `vibeguard W-11`): high / medium / low — derived from the originating agent's evidence.

> This file is the single source of truth for fixing the 72 deduplicated remediation items produced from the 77 raw findings in the 2026-05-01 audit. Each entry includes Fact (cited file:line), Inference (with confidence), Target behavior, Implementation steps, Verification commands, and Risk/assumptions per the Fact–Inference–Suggestion separation rule.

---

## 1. Executive summary

| Severity | Count | Concentration |
|----------|-------|---------------|
| Critical | 20 | Provider data drops, SSRF, SQL-id injection, debug-log PII, unwired pricing, in-memory budgets/rate-limit, parallel pricing/team systems, layer violations, dyn-safety |
| High     | 22 | Cache key gaps, JWT validation, OpenAI-only embeddings/images, 1200+ LOC god files, 35+ unreachable provider dirs, ENV substitution + `deny_unknown_fields` |
| Medium   | 30 | Per-request `reqwest::Client::new()`, MCP rugpull, audit redactor, feature-flag tangles, dead config files |

**Cross-agent confirmations** (highest confidence, treat as P0):

1. Cache key correctness — Agents 1+4 (C20)
2. `reqwest::Client::new()` per-request — Agents 2+4 (H19)
3. JWT secret weak validation — Agents 2+4 (H11)
4. MCP SSRF gap — Agents 2+4 (C7)
5. In-memory budget/rate-limit state — Agents 2+4 (C13, C14)

**Stale memory note**: the entries `crates/* + apps/*` workspace and `litellm-provider-core::DynProvider` referenced in remembered state **do not exist in this checkout**. Either the split was reverted or the memory is from a different branch. C17/C18 specs assume the current single-crate layout.

---

## 2. Phased roadmap

| Phase | Window | Scope | Items | Files | Gate before next phase |
|-------|--------|-------|-------|-------|------------------------|
| **P0** Stop-the-bleeding | Week 1 | Active exploit / silent quota bypass / log PII | C7, C8, C9, C13, C14, C20, H11 | ~12 | All P0 items have a merged PR + green CI; release-tag a `0.5.x` patch |
| **P1** Provider correctness | Week 2–3 | Streaming/non-stream content drops, OpenAI-compat HTTP fields | C1–C6, C10, C19, H1–H8, H19 | ~30 | Integration tests pass for Anthropic stream tool-use, Bedrock tool-result, Gemini tool finish_reason |
| **P2** Architectural consolidation | Week 4–6 | Pricing unification, team-table convergence, Provider source-of-truth | C11, C12, C15, C16, C17, C18, H13–H18, H20–H22 | ~80 | One pricing system, one team-user system, one `ProviderType` table; `cargo check --no-default-features --features lite` passes in CI |
| **P3** Hygiene | Ongoing | M-series + dead code | M1–M30 | ~40 | Quarterly review |

Each phase ends with `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features`, and `bash scripts/guards/check_pr_scope.sh` per CLAUDE.md.

---

## 3. Cross-cutting helpers (build first, reuse everywhere)

Several specs below reuse the same primitives. Build these once before tackling individual findings.

### 3.1 `default_outbound_client()` — shared `reqwest::Client`

**Why** Used by C7, H19, M-series. Fixes timeout-less + per-request-allocation pattern in ~50 current sites.

**Where** New module `src/core/http/outbound.rs`. Re-export from `src/core/http/mod.rs`.

```rust
// src/core/http/outbound.rs
use std::time::Duration;
use std::sync::OnceLock;
use reqwest::{Client, ClientBuilder};

static DEFAULT_CLIENT: OnceLock<Client> = OnceLock::new();

pub fn default_outbound_client() -> &'static Client {
    DEFAULT_CLIENT.get_or_init(|| {
        build_outbound_client(OutboundProfile::default())
            .expect("default outbound client must build")
    })
}

#[derive(Clone, Debug)]
pub struct OutboundProfile {
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
    pub pool_idle_per_host: usize,
    pub user_agent: String,
}

impl Default for OutboundProfile {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(120), // chat completions can run long
            pool_idle_per_host: 32,
            user_agent: format!("litellm-rs/{}", crate::VERSION),
        }
    }
}

pub fn build_outbound_client(profile: OutboundProfile) -> reqwest::Result<Client> {
    ClientBuilder::new()
        .connect_timeout(profile.connect_timeout)
        .timeout(profile.request_timeout)
        .pool_idle_timeout(Some(Duration::from_secs(90)))
        .pool_max_idle_per_host(profile.pool_idle_per_host)
        .user_agent(profile.user_agent)
        .build()
}
```

**Adoption rule**: every `reqwest::Client::new()` site must migrate to either `default_outbound_client().clone()` or a per-provider `OnceLock<Client>` built from `build_outbound_client(...)`. CI guard added in P3.

### 3.2 `is_private_or_reserved_host()` — promote A2A's helper to shared util

**Why** Used by C7 (MCP) and any future outbound-URL validator.

**Where** Move from `src/core/a2a/config.rs:250-309` to `src/core/net/ssrf_guard.rs`. Re-export from both A2A and MCP.

**Surface**:
```rust
pub fn validate_outbound_url(url: &Url) -> Result<(), SsrfError>;
fn is_private_or_reserved_host(host: &str) -> bool;
fn is_private_or_reserved_ip(ip: &IpAddr) -> bool;
```

A future P2 hardening (M7 / DNS rebinding) will add a custom `dns_resolver` on `ClientBuilder` that re-checks resolved IPs.

### 3.3 `ProviderRegistry` — single source of truth for `ProviderType`

**Why** C17, C18, M-series. Today 4 lists drift.

**Where** New file `src/core/providers/registry/types.rs`. Generates `ProviderType` enum, `From<&str>`, `FromStr`, `Display`, support list, catalog-vs-builtin classification from one table:

```rust
#[derive(Clone, Copy, Debug)]
pub enum ProviderKind {
    BuiltinDispatchable, // has a Provider:: variant (OpenAI, Anthropic, Mistral, Cloudflare)
    OpenAILikeWrapped,   // factory wraps with OpenAILikeProvider but still custom branch
    CatalogOnly,         // pure data-driven via PROVIDER_CATALOG
    Stub,                // declared, returns NotImplemented (e.g., PydanticAI)
}

pub struct ProviderEntry {
    pub canonical: &'static str,
    pub aliases: &'static [&'static str],
    pub kind: ProviderKind,
}

pub static PROVIDER_TABLE: &[ProviderEntry] = &[
    ProviderEntry { canonical: "openai", aliases: &["openai-compatible"], kind: BuiltinDispatchable },
    // … one row per provider …
];
```

The enum + `From`/`FromStr`/`Display`/`factory_supported_provider_types()` are all derived (or compile-time generated via `build.rs`/macro) from this table. Tests assert that every catalog entry, every enum variant, and every factory branch is reachable.

### 3.4 `Hasher` policy — deterministic, versioned cache keys

**Why** C20. `DefaultHasher` (SipHash13) is documented as not stable across stdlib versions.

**Where** `src/core/cache/key_generator.rs`. Constants:

```rust
const CACHE_KEY_SCHEMA_VERSION: u32 = 2; // bump on any keying-logic change
fn key_hasher() -> blake3::Hasher { blake3::Hasher::new() }
```

All key builders prepend `CACHE_KEY_SCHEMA_VERSION.to_le_bytes()`. Final hash is `blake3::hash(...).to_hex().to_string()`.

---

## 4. Critical findings — full remediation specs

### C1. Anthropic streaming drops `input_json_delta`, `thinking_delta`, `signature_delta`, `content_block_start`

- **Status**: TODO
- **Severity**: Critical · **Confirmed by**: API agent (high)
- **Files**: `src/core/providers/base/sse.rs:499-526, 592` · `src/core/types/responses/delta.rs:9-34`

**Fact** Today `content_block_delta` arm only matches `delta.text` and emits content; `content_block_start` and `content_block_stop` fall into `_ => Ok(None)`. Tool-use input arguments and extended thinking never reach the client.

**Target behavior**

| Anthropic event | Mapped `ChatDelta` field |
|-----------------|--------------------------|
| `content_block_start { type: "tool_use", id, name, index }` | First chunk for that index: `tool_calls[index] = ToolCallDelta { index, id, type: "function", function: { name, arguments: "" } }` |
| `content_block_delta { delta.type: "input_json_delta", partial_json }` | Append to `tool_calls[index].function.arguments` |
| `content_block_delta { delta.type: "text_delta", text }` | Existing path: `content = Some(text)` |
| `content_block_delta { delta.type: "thinking_delta", thinking }` | `thinking = Some(thinking)` |
| `content_block_delta { delta.type: "signature_delta", signature }` | `thinking_signature = Some(signature)` (new field — see C4) |
| `content_block_stop { index }` | Flush any buffered tool-call args delta with index |
| `message_delta { delta.stop_reason }` | Map per existing code, plus `stop_sequence` (M5) |
| `message_stop` | Emit `[DONE]` per existing path |

**Implementation**

1. Extend `ChatDelta` with `thinking_signature: Option<String>` (depends on C4).
2. Replace `AnthropicTransformer::transform_chunk` body with explicit match on `delta.type`. Use `serde_json::from_value::<AnthropicStreamEvent>(...)` for type safety; no `unwrap_or_default()` on type field (U-23 — fail explicit on unknown types behind a `warn!` once).
3. Per-stream state: `HashMap<u32, ToolCallAccumulator>` carried via `UnifiedSSEParser` state slot. Emit one `tool_calls` chunk on `content_block_start` (with id/name) and one per `input_json_delta`.

**Verification**

- New integration test `tests/anthropic_stream_tool_use.rs` replaying a recorded SSE fixture with both text and tool_use blocks; assert: ≥1 chunk has `tool_calls[0].id`, ≥1 chunk has `tool_calls[0].function.arguments` non-empty, final `finish_reason = "tool_calls"`.
- Fixture stored in `tests/fixtures/anthropic/stream_tool_use.txt`.
- `cargo test --all-features anthropic_stream_tool_use`.

**Risk** [assumption: gateway intends to support Claude 3.5/3.7+ streaming tool use] If a downstream client cannot parse new fields, gate `thinking`/`thinking_signature` behind request flag `include_reasoning: bool` so default behavior preserves OpenAI-shape.

---

### C2. Bedrock Converse drops Tool/Function-role messages and ToolResult content parts

- **Status**: TODO
- **Severity**: Critical · **Confirmed by**: API agent (high)
- **Files**: `src/core/providers/bedrock/chat/converse.rs:241-313`

**Fact** `transform_to_converse` matches only `User | Assistant`; `_ => { /* skip */ }` drops Tool/Function. Tool-result content parts also dropped silently in lines 287-292.

**Target** Translate `MessageRole::Tool` → user message containing `toolResult` block per Bedrock Converse schema:

```json
{ "role": "user",
  "content": [{ "toolResult": { "toolUseId": "<id>", "content": [{ "text": "<result>" }], "status": "success" } }] }
```

**Implementation**

1. Add `MessageRole::Tool` arm constructing `toolResult` JSON. Pull `tool_call_id` from `ChatMessage::tool_call_id` (already present in struct).
2. Add `ContentPart::ToolResult { tool_call_id, content }` arm in the part mapper.
3. Unsupported parts (Image/Audio/Document) — for now return explicit `ProviderError::not_implemented("Bedrock content part Image not yet wired")` rather than silent drop (U-23). M4 will implement these.
4. Add `MessageRole::Function` arm — translate identically to `Tool` (legacy OpenAI `function` is equivalent to `tool`).

**Verification**

- Unit test in `converse.rs` `mod tests` constructing a 3-turn conversation (user, assistant-with-tool_use, tool, assistant) and asserting the JSON matches a hand-written golden.
- Integration test calls Bedrock with a fake tool flow; assert no 400.

**Risk** Bedrock schema differs by foundation model (Claude vs Nova vs Mistral). Initial scope: Claude on Bedrock. Other models tracked under M4 with `not_implemented` placeholder.

---

### C3. Anthropic non-stream drops `thinking` blocks and prompt-cache usage

- **Status**: TODO
- **Severity**: Critical · **Confirmed by**: API agent (high)
- **Files**: `src/core/providers/anthropic/client.rs:578-663`

**Fact** Loop matches only `"text"` and `"tool_use"`; `"thinking"` blocks fall into `_ => {}`. Lines 643-663 read only `input_tokens`/`output_tokens`; cache token counters dropped.

**Target**

- Capture thinking blocks into `ChatMessage.thinking: Option<Thinking>` (struct: `{ content: String, signature: Option<String> }`).
- Map `cache_creation_input_tokens` → `prompt_tokens_details.cache_creation_tokens` (new sub-field).
- Map `cache_read_input_tokens` → `prompt_tokens_details.cached_tokens`.
- Stop-reason mapping: add explicit `"stop_sequence" => FinishReason::StopSequence`. Keep `_ => warn + Stop` only as audit-log fallback (U-23 downgrade path — observable, not silent).

**Implementation**

1. Extend `core::types::responses::Thinking` struct (or add if missing).
2. Extend `PromptTokensDetails` with `cache_creation_tokens: Option<u32>` (already has `cached_tokens`).
3. Add `FinishReason::StopSequence` variant.
4. Replace match block in `transform_response`.

**Verification**

- Unit test using a recorded Anthropic non-stream JSON with thinking + cache hit. Assert `usage.prompt_tokens_details.cache_creation_tokens == 1234` and `assistant.thinking.content` non-empty.
- Update `convert_usage` (chat.rs:479-497) to forward the new sub-fields — covered by H1.

**Risk** Adding a `Thinking` field to `ChatMessage` is a breaking change for any consumer depending on the response shape. Per CLAUDE.md "No backward compatibility — break old formats freely", acceptable.

---

### C4. Public `ChatCompletionDelta` lacks `thinking`/`tool_call_id`/`refusal`/`function_call`

- **Status**: TODO
- **Severity**: Critical
- **Files**: `src/core/streaming/types.rs:81-89` · `src/server/routes/ai/chat.rs:553-560`

**Fact** Internal `ChatDelta` has those fields. The HTTP-facing `ChatCompletionDelta` does not. Conversion drops them.

**Target**

```rust
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ChatCompletionDelta {
    #[serde(skip_serializing_if = "Option::is_none")] pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub thinking: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub thinking_signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub refusal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub tool_calls: Option<Vec<ToolCallDelta>>,
    #[serde(skip_serializing_if = "Option::is_none")] pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub function_call: Option<FunctionCallDelta>,
}
```

**Implementation**

1. Add fields with `skip_serializing_if` (also addresses C19).
2. Update `convert_core_chunk_to_streaming` to forward them.
3. `ToolCallDelta` already has the right shape; no change needed.

**Verification**

- `cargo test --all-features streaming::types` asserts JSON serialization omits `null`s.
- Update or add `chat.rs` test forwarding a `ChatDelta { thinking: Some(...) }` and asserting it survives.

---

### C5. Gemini tool-call responses report `finish_reason: Stop` instead of `ToolCalls`

- **Status**: TODO · **Severity**: Critical · **Files**: `src/core/providers/gemini/client.rs:469-510`

**Fact** Stream + non-stream both map `STOP` → `Stop` even when `tool_calls` were extracted (line 457-464).

**Target** After collecting `tool_calls`: if non-empty AND `content` is empty → override `finish_reason = ToolCalls`.

**Implementation** ~6 lines after the `tool_calls` extraction loop.

**Verification** Unit test using a Gemini fixture where the candidate has only `functionCall` parts — assert `choice.finish_reason == "tool_calls"`.

---

### C6. LiteLLM-compat helper drops `tools`/`tool_choice`/`response_format`

- **Status**: TODO · **Severity**: Critical · **Files**: `src/core/completion/conversion.rs:11-46` · `src/core/completion/types.rs:41-43`

**Fact** Conversion hardcodes `tools: None, tool_choice: None, parallel_tool_calls: None, response_format: None` even though `CompletionOptions` carries `tools` and `tool_choice`.

**Target**

1. Forward `options.tools` and `options.tool_choice`.
2. Extend `CompletionOptions` with `parallel_tool_calls`, `response_format`, `thinking`, `reasoning_effort`, `metadata`, `service_tier`, `store`, `prediction` (covers C6 + M3).
3. Forward all of them.

**Verification** `cargo test --all-features completion::conversion::tests` — add a test passing a tool and asserting it appears in the resulting `ChatRequest`.

---

### C7. MCP transport URLs lack SSRF guard ⭐ (Agents 2+4)

- **Status**: TODO · **Severity**: Critical · **Files**: `src/core/mcp/config.rs:128-157`

**Fact** Validates only scheme prefix. A2A code (a2a/config.rs:231-309) has full guard against loopback/RFC1918/IMDS — MCP doesn't reuse it.

**Target** Use the shared helper from §3.2 (`validate_outbound_url`) inside `ServerConfig::validate()`.

**Implementation**

1. Build §3.2 helpers first.
2. In `mcp/config.rs::validate()`, after scheme check, call `validate_outbound_url(&parsed)?`.
3. Add downgrade path: env var `LITELLM_MCP_ALLOW_PRIVATE_TARGETS=1` permits private targets for local-dev use, with a startup `warn!` documenting the bypass (U-32 observable downgrade path).

**Verification**

- New tests in `mcp/config.rs::tests`: 169.254.169.254 rejected, localhost rejected, public host accepted, env-var bypass accepted with warning logged.
- Manual: `curl -X POST /admin/mcp/servers -d '{"url":"http://169.254.169.254/..."}'` returns 4xx.

**Risk** Some local-dev MCP setups legitimately point at `localhost`. The env-var bypass covers them. Default = secure.

---

### C8. pgvector raw-SQL identifier interpolation

- **Status**: TODO · **Severity**: Critical · **Files**: `src/core/providers/pg_vector/provider.rs:208-252, 443-451` · `config.rs:317-318`

**Fact** `format!()` injects `schema`, `table_name`, `threshold`, `limit` straight into SQL. Current `PgVectorConfig::validate()` rejects most special characters, but it allows identifier rules to live separately from SQL construction; `to_sql_string` rolls hand-written `'` escaping; `full_table_name()` does not escape embedded `"`.

**Target** Strict identifier validation + parameterized values:

1. In `PgVectorConfig::validate()`, reject schema/table_name not matching `^[A-Za-z_][A-Za-z0-9_]{0,62}$`.
2. Replace `format!(" LIMIT {}", options.limit)` with bind parameter.
3. Replace `to_sql_string` with a call to driver's prepared-statement API; remove hand-rolled `'` escape.
4. Threshold/operator stay literal but validate operator against `&[<=, <#, <->]` enum.

**Implementation** Audit every `format!` call in `pg_vector/`; change to bind params or validated enums. Delete `to_sql_string`.

**Verification**

- Unit test: `with_table_name(r#"users"; DROP TABLE users--"#)` returns validation error.
- Integration test against a real Postgres: queries with malicious bind values do not execute injected SQL.
- `cargo audit` for the SQL driver.

---

### C9. `debug!` logs full request bodies (PII / secrets)

- **Status**: TODO · **Severity**: Critical · **Files**: `src/core/providers/bedrock/client.rs:169-170` · `src/core/providers/milvus/provider.rs:239` · audit all `debug!.*body` patterns

**Fact** `debug!("Request body: {}", body_str)`. The audit redactor (`audit/config.rs:111-116`) does not run through `tracing`.

**Target** Body content goes to `trace!`, gated by feature `debug-request-bodies` (off by default). Replace with redacted summary at `debug!`:

```rust
debug!(
    "Bedrock request: {} to {} (body_size={}B, message_count={}, tool_count={})",
    operation, url, body_size, message_count, tool_count
);
#[cfg(feature = "debug-request-bodies")]
trace!("Bedrock body: {}", redact(&body_str));
```

**Implementation**

1. Add Cargo feature `debug-request-bodies` (default off).
2. Add `redact()` shim in `core::audit::redaction` that runs the configured redactor's regex set on a string.
3. Audit all `debug!` / `info!` macros that include `body`, `request`, `response`, `Authorization`, `messages`, `prompt`. Apply same treatment.
4. Add a CI guard script `scripts/guards/check_log_pii.sh` grepping for `debug!.*body|info!.*body` and failing on additions outside the feature gate.

**Verification**

- `RUST_LOG=debug cargo run` against a prompt with `sk-...` content; verify the literal does not appear in stdout.
- `cargo build --features debug-request-bodies` succeeds and emits trace logs only when configured.

---

### C10. `core::streaming::mod.rs` imports `actix_web` (layer violation)

- **Status**: TODO · **Severity**: Critical · **Files**: `src/core/streaming/mod.rs:7-34`

**Target** Move `create_sse_response` and any `HttpResponse` builder to `src/server/sse.rs`. `core::streaming` exposes only framework-agnostic `Stream<Item = Result<Bytes, ProviderError>>` builders.

**Implementation**

1. Create `src/server/sse.rs` with the actix-touching code.
2. Strip imports from `core/streaming/mod.rs`. The few helpers that need `actix_web::http::header` constants — define our own `pub const SSE_CONTENT_TYPE: &str = "text/event-stream"` instead.
3. Update callers (`src/server/routes/ai/chat.rs`, `responses_stream.rs`).
4. Re-run `cargo check --no-default-features` to confirm `core` is framework-clean.

**Verification** `rg "actix_web" src/core` returns 0 hits.

---

### C11. 3–4 parallel pricing systems

- **Status**: TODO · **Severity**: Critical · **Files**: `src/core/cost/calculator.rs` (1645 LOC) · `src/core/providers/base/pricing.rs` · `src/services/pricing/` · `src/core/providers/anthropic/mod.rs:24-27`

**Decision** `services::pricing::PricingService` is the canonical owner because it is the one stored on `AppState` (`http.rs:48-52`). Delete or thin-wrap the other three.

**Implementation (sequenced — do not parallelize)**

1. Migrate `core::cost::CostCalculator::calculate_*` callers to `PricingService::calculate(...)`. Maintain trait surface temporarily.
2. Delete `core::providers::base::pricing` (`PricingDatabase`, `get_pricing_db`). `Provider::calculate_cost` now reads `app_state.pricing_service` via injected handle (extend `Provider` API to accept `&PricingService` or a `RequestContext`).
3. Delete `anthropic::ModelPricing` / `anthropic::CostCalculator` re-exports.
4. Delete `core::providers::ModelPricing` (mod.rs:218-226) per M1.
5. Move data files: keep `config/model_prices_extended.json`. Delete `config/pricing.yaml` per H18 (or implement loading — see below).

**Verification**

- `rg "ModelPricing" src/` returns one struct definition (in `services::pricing`).
- All cost-related tests pass.
- New integration test: same model on Together vs DeepInfra returns different cost (also fixes M2 — provider name is keyed).

**Risk** [assumption: callers do not depend on the deleted public types] If external SDK users imported `litellm_rs::core::cost::ModelPricing`, this is a SemVer break. Per CLAUDE.md, acceptable in 0.x.

---

### C12. Pricing migration declared but unregistered; phantom `pricing_history` entity

- **Status**: TODO · **Severity**: Critical · **Files**: `src/storage/database/migration/m20240201_000001_create_pricing_tables.rs` · `migration/mod.rs:3-27` · `entities/pricing.rs:43,47,90,97,103,107` · `entities/mod.rs:1-19`

**Decision** Pricing tables stay (will eventually back budget persistence — see C13). Wire them in.

**Implementation**

1. Add `mod m20240201_000001_create_pricing_tables;` in `migration/mod.rs` and register in `MigratorTrait::migrations()`.
2. Create `src/storage/database/entities/pricing_history.rs` with the entity referenced by `pricing.rs`.
3. Add `pub mod pricing; pub mod pricing_history;` in `entities/mod.rs`.
4. Run `cargo check --all-features` — module references must compile.
5. Run `gateway database migrate` against a clean DB — assert tables exist (`\d model_pricing`).

**Verification** Add a `migration_smoke_test.rs` using `sqlx::SqlitePool` that runs all migrations and selects from `model_pricing` and `pricing_history`.

---

### C13. Budgets in-memory only ⭐ (Agents 2+4)

- **Status**: TODO · **Severity**: Critical · **Files**: `src/server/state.rs:64` · `src/core/budget/provider_limits.rs:105-107, 640-660`

**Decision** Persist via Postgres. Memory remains the hot-path read-through cache.

**Implementation**

1. Add `budget_spend` table schema (migration). Columns: `(scope_type, scope_id, provider, model, period_start, period_end, spend_usd, updated_at, version)`. Unique on `(scope_type, scope_id, provider, model, period_start)`.
2. Extend `UnifiedBudgetLimits` with `pub async fn save(&self, store: &DatabaseStore) -> Result<()>` and `pub async fn restore(store: &DatabaseStore) -> Result<Self>`.
3. Call `restore()` in `AppState::new_with_unified_router` BEFORE serving traffic.
4. Periodic flush every 30s + on graceful shutdown (`actix_web::dev::Server::stop` hook).
5. On request: read from in-memory; write deltas into a small ring buffer that the flusher drains.

**Verification**

- Integration test: spend $5, restart process, assert `current_spend == $5`.
- Multi-replica test: two replicas writing simultaneously, no double-count beyond the flush window.

**Risk** Cross-replica sub-second consistency requires either a transactional spend table or Redis. P0 ships Postgres-only with documented eventual-consistency window of ≤30s. Hard guarantees deferred to follow-up.

---

### C14. Rate limiter per-process DashMap ⭐ (Agents 2+4)

- **Status**: TODO · **Severity**: Critical · **Files**: `src/core/rate_limiter/limiter.rs:5-33`

**Decision** Implement a Redis-backed strategy when `redis.enabled = true`. Fail-closed for HA: if `redis.enabled = false` and `replicas > 1` (env var or detected), refuse to start with `enable_rate_limit: true`.

**Implementation**

1. New `RateLimiter` trait with two impls: `InProcessRateLimiter` (current DashMap) and `RedisRateLimiter` (Lua token-bucket).
2. Selection at `AppState::new` based on config.
3. Lua script atomic increment + window-rollover. Store under key `rl:{tenant}:{minute_window}`.
4. Add unbounded-cardinality fix: cap DashMap at N entries with LRU eviction (also addresses H9).

**Verification**

- Multi-replica integration test: 2 replicas, 1000 RPM cap, 100 RPS load → exactly 1000 allowed in any 60s window.
- Single-replica: existing tests still pass.

---

### C15. Two parallel team/user systems

- **Status**: TODO · **Severity**: Critical · **Files**: `m20240301_000001_create_user_management_tables.rs` (um_*) · `m20240301_000002_create_teams_table.rs` (teams) · `seaorm_db/user_management_ops.rs` · `team_repository.rs`

**Decision** SeaORM-based `users`/`teams` is canonical (matches the rest of the storage layer). Migrate `um_*` data, delete `um_*` tables in a follow-up migration after data is preserved.

**Implementation (sequenced, must not interleave)**

1. Add migration `m20260501_000001_migrate_um_to_canonical.rs` that copies rows from `um_users → users`, `um_teams → teams`, `um_organizations → organizations` (create the latter if missing).
2. Update `user_management_ops.rs` to read/write from `users`/`teams` via SeaORM.
3. Add a guarded second migration `m20260501_000002_drop_um_tables.rs` (manual approval gate — see "Publish/destructive confirmation" rule below).
4. Delete `user_management_ops.rs` once `team_repository` covers all use cases (verify with `rg "user_management_ops"`).

**Risk** Existing deployments may have data in `um_*`. The two-migration approach (copy first, drop later) is reversible until step 3 runs. Step 3 is a destructive op — apply the four-point confirmation per `vibeguard W-10` before merge.

---

### C16. `AuthConfig::default()` returns invalid state (`enable_jwt: true, jwt_secret: ""`)

- **Status**: TODO · **Severity**: Critical · **Files**: `src/config/models/auth.rs:51-63`

**Target**

```rust
impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enable_jwt: false,           // require explicit opt-in
            enable_api_key: true,
            jwt_secret: String::new(),
            // …
        }
    }
}
```

Plus: change `jwt_secret: String` to `jwt_secret: Option<String>` so the type forces awareness. `validate()` requires `Some(s)` whenever `enable_jwt`.

**Verification** `cargo test config::auth::default_round_trip_through_validate` — the Default value passes validation.

---

### C17. `LLMProvider` not dyn-safe; 4-arm macro dispatch over 5-variant enum

- **Status**: TODO · **Severity**: Critical (architectural) · **Files**: `src/core/traits/provider/llm_provider/trait_definition.rs:42-43` · `src/core/providers/mod.rs:243-313, 340-347`

**Decision** Keep enum dispatch (perf-sensitive hot path) but generate the macro arms from §3.3 `PROVIDER_TABLE`. Do not migrate to `Box<dyn LLMProvider>` — the `async fn in trait` decision was deliberate; reverting would require `#[async_trait]` and trait objects throughout.

**Implementation**

1. Build §3.3 first.
2. Replace the 4-arm `dispatch_provider!` body with one driven by `proc_macro` or `macro_rules!` over `PROVIDER_TABLE`.
3. Add CI guard: `cargo expand` snapshot of `dispatch_provider!` to catch silent drift.

**Verification** `cargo test --all-features` plus `cargo expand` diff is empty on no-op changes.

---

### C18. Four sources of truth for "supported providers" drift

- **Status**: TODO · **Severity**: Critical · **Files**: `src/core/providers/{mod.rs,provider_type.rs,factory/registry.rs,registry/catalog.rs}`

Resolved by §3.3. After §3.3 lands:

1. `ProviderType` enum is generated.
2. `From<&str>` / `FromStr` / `Display` are derived from `aliases`.
3. `factory_supported_provider_types()` returns the set of `canonical` names with `kind != Stub`.
4. The catalog stays as data — `kind: CatalogOnly` rows reference catalog entries.

Add test: every alias resolves; every enum variant has a `Provider` variant or factory branch or is `Stub`. No "decoded but unbuildable" combinations.

---

### C19. Streaming chunk types miss `skip_serializing_if`

- **Status**: TODO · **Severity**: Critical · **Files**: `src/core/streaming/types.rs:50-89`

**Target** Already addressed in C4 by replacing the struct definition. Apply the same `#[serde(skip_serializing_if = "Option::is_none")]` to:

- `ChatCompletionChunk` — `usage`, `system_fingerprint`, `service_tier`
- `ChatCompletionChunkChoice` — `logprobs`, `finish_reason`
- `ChatCompletionDelta` — all fields (per C4)

**Verification** Unit test: serialize a chunk with only `content = Some("x")` and assert the JSON has exactly `{ id, object, created, model, choices: [{ index: 0, delta: { content: "x" } }] }` and nothing else.

---

### C20. Cache key uses `DefaultHasher`; misses tool params, schema, code-version ⭐ (Agents 1+4)

- **Status**: TODO · **Severity**: Critical · **Files**: `src/core/cache/key_generator.rs:30-90, 152-170`

Resolved via §3.4 helpers. Detail:

1. Replace `DefaultHasher` with `blake3::Hasher`.
2. Hash domain prefix: `cache:v{CACHE_KEY_SCHEMA_VERSION}:chat:` / `cache:v{...}:embed:` etc.
3. Hash full normalized JSON of tools (`tool.function.{name,description,parameters,strict}`), `tool_choice`, `response_format` (including `json_schema`), `parallel_tool_calls`, `reasoning_effort`, `service_tier`, `logit_bias`.
4. Hash `tool_calls`, `tool_call_id`, `function_call` on assistant messages.

**Verification**

- Unit test: two requests differing only in `tool.function.parameters` → different keys.
- Unit test: bumping `CACHE_KEY_SCHEMA_VERSION` → all keys shift.

**Migration** Bumping the version on first deploy invalidates the existing Redis cache (acceptable cold start).

---

## 5. High findings — compact specs

> Format: ID · severity · agent · file:line · target · verify

### Provider contract (H1–H8)

**H1** `convert_usage` drops `thinking_usage` · `chat.rs:479-497` · forward `thinking_usage` and the new `cache_creation_tokens` from C3 · cargo test usage round-trip.

**H2** `parallel_tool_calls`/`extra_body`/`prediction`/`safety_settings`/`cache_control` not exposed at HTTP boundary · `src/core/models/openai/requests.rs:14-110` · add fields with `#[serde(default)]` and forward in `build_core_chat_request`. Add `#[serde(flatten)] extra: HashMap<String, Value>` to absorb provider-specific knobs (also fixes M2).

**H3** `seed: u32` cast to `i32` wraps · `chat.rs:373` · widen internal `ChatRequest.seed` to `i64`; remove the `as i32`.

**H4** Anthropic transform overwrites assistant text content when tool_calls exist · `anthropic/client.rs:480-498` · append tool_use blocks instead of replacing.

**H5** Anthropic Tool-role messages emitted as plain user text · `anthropic/client.rs:399-401` · emit `[{ type: "tool_result", tool_use_id: msg.tool_call_id, content: text }]`.

**H6** `response_type` field always None · `chat.rs:334-340` · either thread it through or delete (U-05). Per audit's H7, mark DEFER until provider consumers are confirmed.

**H7** Catalog guard ordering can shadow Tier-2 branches · `factory/registry.rs:69, 99-175` · move catalog guard to the end of the match (right before wildcard) OR delete duplicate explicit branches. Add unit test: each Tier-2 enum variant constructs the right `Provider` variant.

**H8** `Provider::create_embeddings/create_images` only OpenAI · `mod.rs:481-512` · wire all variants through `dispatch_provider!`. For providers without the capability, return `not_implemented` per-variant (preserves U-23 explicit failure).

### Security & error handling (H9–H12)

**H9** Unbounded auth-rate-limiter DashMap · `auth_rate_limiter.rs:8-13` · cap at N entries with LRU eviction; run `cleanup_old_entries` from a tokio background task started in `HttpServer::new` (independent of auth handlers).

**H10** `expect("invalid CORS configuration ...")` in `create_app` · `http.rs:113` · replace with match returning previous app factory + `error!`. Validate CORS at config-load time and refuse to swap.

**H11** ⭐ JWT lowercase-only secret accepted; example value passes validate · `auth.rs:94-117` + `gateway.yaml.example:101` · (a) raise lowercase-only to hard error, (b) deny-list any string containing `"Replace"`, (c) change example to `${LITELLM_JWT_SECRET}`.

**H12** Audit logger silent shutdown drops · `audit/logger.rs:131-132, 166-169` · replace `let _ = ` with `if let Err(e) = ... { error!(target: "audit", ...) }` and bump a Prometheus counter on dropped events.

### Architecture / dead code (H13–H17)

**H13** 35+ provider directories unreachable from factory · `src/core/providers/mod.rs:11-177` · for each: decide wire-or-delete. Default: delete unless owner steps up. Add CI guard (P3): every `src/core/providers/<name>/mod.rs` must be referenced from `Provider::from_config_async` or fail CI.

**H14** SDK reimplements router/load-balancer · `src/sdk/client/routing.rs` · replace `LoadBalancer` with a thin wrapper around `core::router::UnifiedRouter`. Single test asserts `LLMClient::send` and `completion()` produce the same provider selection on a fixed seed.

**H15** God files (`cost/calculator.rs` 1645, `anthropic/{models,client}.rs` 1268/1152, `base/sse.rs` 1251) · split per the breakdown in audit report §"Top 10 god files". Sequence after C11 (which deletes ~half of `cost/calculator.rs`).

**H16** `src/server/handlers.rs` empty placeholder · delete file and `mod handlers;` line in `server/mod.rs:11`.

**H17** `ProviderType::PydanticAI` declared, never implemented, used as test fixture · `provider_type.rs:36` · either delete the variant + aliases OR implement. Removing the test as fixture material — find another "unsupported" placeholder.

### Config & persistence (H18–H22)

**H18** `config/pricing.yaml` never loaded · delete the file, or implement loading (see C11). Default: delete in P2.

**H19** ⭐ ~50 `reqwest::Client::new()` sites outside obvious tests · migrate every site to §3.1 `default_outbound_client()` or a per-provider `OnceLock<Client>`. CI guard grepping for new occurrences (P3).

**H20** `${ENV_VAR}` substitution silently leaves placeholder · `src/config/mod.rs:32-57` · after substitution, scan for any remaining `${[A-Za-z_]\w*}` and refuse to start with the missing-variable list. Add `ConfigError::UnresolvedEnvVars(Vec<String>)`.

**H21** No `#[serde(deny_unknown_fields)]` · add to all top-level config structs and major sub-structs. Run a config-lint test that round-trips `gateway.yaml.example` and asserts no unknown keys.

**H22** Default pricing path resolution diverges from example · `gateway.rs:228-242` · adopt one resolver: `LITELLM_PRICING_SOURCE` → `<data_local_dir>/litellm-rs/model_prices_extended.json` → `./config/model_prices_extended.json`. Document in `gateway.yaml.example`.

---

## 6. Medium findings — reference table

| ID | File:Line | Action | Effort |
|----|-----------|--------|--------|
| M1 | `chat.rs:562-564` | Replace `.ok()` with `unwrap_or_else + error!` | XS |
| M2 | `requests.rs:14-75` | Solved by H2 (`extra` flatten) | — |
| M3 | `completion/conversion.rs:40-44` | Solved by C6 | — |
| M4 | `bedrock/chat/converse.rs:263-298` | Implement Image/Audio/Document parts; in interim, return `not_implemented` (already in C2) | M |
| M5 | `anthropic/client.rs:633-637` | Add `FinishReason::StopSequence` (covered in C3) | — |
| M6 | `gemini/client.rs:514-530` | Map `cachedContentTokenCount`, `thoughtsTokenCount` | S |
| M7 | `a2a/config.rs:250-309` | Custom `dns_resolver` rechecks resolved IPs | M |
| M8 | `mcp/server.rs:49-50, 187-217` | Hash tool descriptions on first connect; diff on refresh; reject changes (SEC-12) | M |
| M9 | `audit/config.rs:111-116` | Add patterns: `gw-`, `eyJ...JWT`, `AKIA`, `Bearer`, `sk-ant-`, full Authorization-header redaction | S |
| M10 | `auth/password.rs:50-56` | Constant-time path: always run dummy work; sleep to fixed duration | S |
| M11 | `secret_managers/file.rs:117-125` | Move `validate_path` before `path.exists()` | XS |
| M12 | `openai_like/provider.rs:406` | Generic message + correlation id; full body to server-side log only | S |
| M13 | `database/mod.rs:23` vs `files/mod.rs:23` | Introduce `LITELLM_HOME`; derive `LITELLM_SQLITE_PATH`/`LITELLM_DATA_DIR` from it | S |
| M14 | `state.rs:21-26` | Add `notify`-based watcher; or downgrade comment + remove `AtomicValue` until shipped | M |
| M15 | `gateway.yaml.example:90-95` | Document which subsystems honor Redis; runtime warning when `redis.enabled` but consumer can't use it | XS |
| M16 | `mod.rs:218-226` | Delete struct + 4 self-tests | XS |
| M17 | `mod.rs:449-463` | Solved by C11 (PricingService keyed by `(provider, model)`) | — |
| M18 | `openai_like/provider.rs:46-51` | Change trait to `&str` lifetime-of-self or `Cow<'static, str>` | S |
| M19 | `mcp/mod.rs:73-76` | Rename `AuthType`/`PermissionLevel` to `Mcp*` prefixes | S |
| M20 | `Cargo.toml:155-174` | Add CI matrix: `cargo check --no-default-features --features lite` | XS |
| M21 | `Cargo.toml:188-197` | Split `metrics-system` (sysinfo) from `metrics-request`; `analytics` only depends on the latter | S |
| M22 | `Cargo.toml:195` | Re-add `qdrant-client` behind feature, or rename to clarify code-only flag | S |
| M23 | `core/providers/macros/` | Audit usage of each macro; delete unused ones; document remaining in `MACROS.md` | M |
| M24 | `Cargo.toml:52` | Drop `paste = pastey` rename; import as `pastey` everywhere | S |
| M25 | `lib.rs:90-134` | Curated `prelude` mod; mark less-stable types `#[doc(hidden)]` | M |
| M26 | `gateway.rs:228-242` | Solved by H22 | — |
| M27 | `m20240301_000001` + `m20240301_000002` | Solved by C15 | — |
| M28 | `core/streaming/providers.rs` | Delete; consolidate on `base/sse.rs` transformers | M |
| M29 | `key_generator.rs` | Solved by C20 (blake3 + version prefix) | — |
| M30 | various | Solved by §3.1 `default_outbound_client` | — |

---

## 7. Verification matrix

Each phase ends green only if all rows pass:

| Gate | Command | Phase |
|------|---------|-------|
| Format | `cargo fmt --all -- --check` | every |
| Lint | `cargo clippy --all-targets --all-features -- -D warnings` | every |
| Unit + integration | `cargo test --all-features` | every |
| Lite build | `cargo check --no-default-features --features lite` | P2+ |
| Layer-violation | `rg "actix_web\|sea_orm\|redis::" src/core` returns 0 | P1 (after C10) |
| Per-request HTTP client | `rg "reqwest::Client::new\(\)" src/` returns ≤ 3 hits (only `core/http/outbound.rs`) | P2 (after H19) |
| Single pricing system | `rg "ModelPricing\b" src/` lists exactly 1 struct definition | P2 (after C11) |
| Single team table | `rg "um_users\|um_teams" src/` returns 0 | P2 (after C15) |
| Cache key version | New blake3 + `CACHE_KEY_SCHEMA_VERSION` test passes | P0 (C20) |
| MCP SSRF | `cargo test mcp::config::ssrf` passes | P0 (C7) |
| Anthropic stream tool-use | `cargo test anthropic_stream_tool_use` passes | P1 (C1) |
| Bedrock tool-result round-trip | `cargo test bedrock_tool_result_round_trip` passes | P1 (C2) |
| Budget persistence | Spend $5, restart, assert $5 | P0 (C13) |
| Multi-replica rate-limit | Lua-bucket test passes | P0 (C14) |
| PR scope | `bash scripts/guards/check_pr_scope.sh` | every |
| PR overlap | `bash scripts/guards/check_pr_overlap.sh` | every |

---

## 8. Tracking table

> Per CLAUDE.md "one issue → one branch → one PR". Use this as the master tracker. Update `Status` and `PR` fields as work lands.

| ID | Title | Severity | Phase | Effort | Owner | PR | Status |
|----|-------|----------|-------|--------|-------|----|----|
| C1 | Anthropic streaming tool/thinking deltas | Critical | P1 | M | - | - | TODO |
| C2 | Bedrock Tool-role / ToolResult | Critical | P1 | M | - | - | TODO |
| C3 | Anthropic non-stream thinking + cache usage | Critical | P1 | S | - | - | TODO |
| C4 | `ChatCompletionDelta` missing fields | Critical | P1 | S | - | - | TODO |
| C5 | Gemini tool finish_reason | Critical | P1 | XS | - | - | TODO |
| C6 | LiteLLM helper drops tools | Critical | P1 | S | - | - | TODO |
| C7 | MCP SSRF guard ⭐ | Critical | P0 | S | - | - | TODO |
| C8 | pgvector SQL identifier | Critical | P0 | M | - | - | TODO |
| C9 | debug-log PII | Critical | P0 | S | - | - | TODO |
| C10 | core::streaming actix import | Critical | P1 | S | - | - | TODO |
| C11 | Unify pricing systems | Critical | P2 | L | - | - | TODO |
| C12 | Wire pricing migration + entity | Critical | P2 | S | - | - | TODO |
| C13 | Persist budgets ⭐ | Critical | P0 | M | - | - | TODO |
| C14 | Redis-backed rate limit ⭐ | Critical | P0 | M | - | - | TODO |
| C15 | Converge team/user systems | Critical | P2 | L | - | - | TODO |
| C16 | `AuthConfig::default` invalid | Critical | P2 | XS | - | - | TODO |
| C17 | Provider dispatch macro source | Critical | P2 | M | - | - | TODO |
| C18 | Single ProviderType source of truth | Critical | P2 | M | - | - | TODO |
| C19 | Streaming chunk skip_serializing_if | Critical | P1 | XS | - | - | TODO |
| C20 | Cache key blake3 + version ⭐ | Critical | P0 | S | - | - | TODO |
| H1 | Forward thinking_usage | High | P1 | XS | - | - | TODO |
| H2 | Expose parallel_tool_calls / extra | High | P1 | S | - | - | TODO |
| H3 | seed widen to i64 | High | P1 | XS | - | - | TODO |
| H4 | Anthropic content+tool_calls coexist | High | P1 | XS | - | - | TODO |
| H5 | Anthropic tool_result blocks | High | P1 | S | - | - | TODO |
| H6 | response_type plumbing or DEFER | High | P1 | XS | - | - | DEFER |
| H7 | Catalog guard ordering | High | P1 | XS | - | - | TODO |
| H8 | Embeddings/images per provider | High | P1 | M | - | - | TODO |
| H9 | Auth rate limiter cap + bg cleanup | High | P0 | S | - | - | TODO |
| H10 | CORS expect → graceful fallback | High | P1 | XS | - | - | TODO |
| H11 | JWT secret hard-rules ⭐ | High | P0 | XS | - | - | TODO |
| H12 | Audit logger error reporting | High | P1 | XS | - | - | TODO |
| H13 | 35+ orphan provider dirs | High | P2 | L | - | - | TODO |
| H14 | SDK delegate to UnifiedRouter | High | P2 | M | - | - | TODO |
| H15 | Split god files | High | P2 | L | - | - | TODO |
| H16 | Delete empty `handlers.rs` | High | P2 | XS | - | - | TODO |
| H17 | PydanticAI: implement or remove | High | P2 | S | - | - | TODO |
| H18 | Delete `pricing.yaml` | High | P2 | XS | - | - | TODO |
| H19 | Shared outbound HTTP client ⭐ | High | P1 | M | - | - | TODO |
| H20 | Fail on unresolved ${ENV} | High | P2 | S | - | - | TODO |
| H21 | deny_unknown_fields | High | P2 | M | - | - | TODO |
| H22 | One pricing-source resolver | High | P2 | XS | - | - | TODO |
| M1–M30 | (see §6) | Medium | P3 | varies | - | - | TODO |

---

## 9. Operational rules during remediation

Per CLAUDE.md where cited, plus this audit campaign's hygiene rules:

1. **One PR per ID** in this table. Branch name `fix/audit-{ID}` (e.g. `fix/audit-C7`).
2. **Always branch from latest `main`**. Run `bash scripts/guards/check_pr_overlap.sh` before pushing.
3. **Max 10 files / 500 lines per PR** excluding Cargo.lock and docs.
4. **Commits are DCO-signed** (`Signed-off-by: ...`) for this campaign. **No `Co-Authored-By` and no AI markers** unless the owner changes that rule.
5. **No `--no-verify`**. If a hook fails, fix the underlying issue.
6. **Destructive migrations** (C12 step 3, C15 step 3, H13 deletions) require the four-point confirmation in the PR description before merge.
7. **Verification evidence** in every PR body: paste the relevant test command output (W-16 — fresh, in-PR-CI evidence, not "passed earlier").

---

## 10. Open questions for owner decisions

| Q | Where | Decision needed before |
|---|-------|------------------------|
| Q1 | Is `H13` "delete 35+ provider dirs" acceptable, or are some provider dirs in-progress? | Sprint planning for P2 |
| Q2 | C13 ships eventual-consistency budgets (≤30s lag). Is that acceptable, or do we need synchronous Postgres write per request (latency hit)? | C13 implementation |
| Q3 | C15 step 3 drops `um_*` tables. Confirm no production deployment depends on raw SQL queries against those names. | C15 implementation |
| Q4 | C17 keeps enum dispatch. Confirm performance budget vs simplicity of `Box<dyn>`. | C17 implementation |
| Q5 | Which provider dirs in H13 are wanted Tier 2 (gemini, cohere are likely yes)? | P2 kickoff |

---

## 11. Source audit reports

The original raw findings (with full evidence quotes) from each agent are preserved in the chat transcript that produced this spec. If they need to be persisted, copy them to `docs/audit-2026-05-01/`:

- `agent-1-api-data-integrity.md` — 22 findings (7C/8H/7M)
- `agent-2-error-security.md` — 15 findings (3C/5H/7M)
- `agent-3-architecture.md` — 22 findings (4C/8H/10M)
- `agent-4-config-persistence.md` — 18 findings (6C/6H/6M)

Raw total: 77 findings, 20 Critical, 27 High, 30 Medium.

Deduplicated execution tracker: 72 remediation items, 20 Critical, 22 High, 30 Medium. Treat §8 as the executable source of truth; raw agent counts are retained only for provenance.
