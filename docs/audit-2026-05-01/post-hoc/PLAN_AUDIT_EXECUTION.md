# Audit Remediation — Execution Plan

> **ARCHIVED — 2026-05-06.** This post-hoc execution plan was authored from chat-side analysis. The canonical record is [`docs/plan/audit-remediation-complete-plan.md`](../../plan/audit-remediation-complete-plan.md); closure status is in [`closeout-2026-05-02.md`](../closeout-2026-05-02.md). All 41 remediation steps closed on 2026-05-02 via PR #463–#495. This file is kept for cross-reference only and is not a live tracker.

> Companion to `PLAN_AUDIT_REMEDIATION.md`. The spec describes *what* each fix is. This plan describes the *order*, *dependencies*, *parallelism*, and *daily rhythm* for landing the 72 deduplicated remediation items from the 77 raw audit findings.

**Total scope**: 20 Critical + 22 High + 30 Medium = 72 deduplicated remediation items.
**Target horizon**: P0 in week 1, P0+P1 in 3 weeks, all Critical+High in 6 weeks, full set in 10 weeks.
**Working assumption**: 1 primary engineer + occasional parallel agents (Codex / harness-delegate) for independent quick wins.

---

## 0. Plan-at-a-glance

```
Week 1     Week 2-3            Week 4-6              Week 7-10
──────     ─────────           ──────────            ─────────
Wave 0  →  Wave 3 (provider    Wave 4 (architectural Wave 5
(founda-   correctness, P1)    consolidation, P2)    (hygiene,
tions)                                               P3 medium)
   ↓
Wave 1
(quick wins)
   ↓
Wave 2
(P0 critical)
```

| Wave | Window | What | Items | Parallelism |
|------|--------|------|-------|-------------|
| 0 | Day 1–2 | Build cross-cutting helpers (spec §3) | 4 helpers | sequential (each landed before next) |
| 1 | Day 1–3 (overlaps Wave 0) | Quick wins, no helper dependency | ~10 small fixes | high (parallel agents OK) |
| 2 | Day 3–7 | P0 critical, depend on Wave 0 helpers | C7, C8, C9, C13, C14, C20, H9, H11 | medium (some shared files) |
| 3 | Week 2–3 | P1 provider correctness | C1–C6, C10, C19 + H1–H8, H10, H12, H19 | high after C4 lands |
| 4 | Week 4–6 | P2 architectural consolidation | C11, C12, C15, C16, C17, C18, H13–H18, H20–H22 | low (sequential refactors) |
| 5 | Week 7+ | P3 hygiene | M1–M30 | high |

---

## 1. Kickoff checklist (do once, before Wave 0)

Run through this list before landing the first fix PR.

```bash
# 1. Confirm clean working tree
git status                                # must be clean
git fetch origin && git checkout main
git pull --ff-only origin main

# 2. Confirm baseline build is green
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features                 # snapshot the pass count
cargo check --no-default-features --features lite  # may fail today; that's OK, M20 fixes it

# 3. Snapshot current god-file LOC for later progress check
wc -l src/core/cost/calculator.rs \
      src/core/providers/anthropic/{models,client}.rs \
      src/core/providers/base/sse.rs \
      src/core/providers/openai_like/provider.rs \
      src/server/routes/ai/chat.rs \
      src/core/providers/factory/builder.rs > /tmp/audit-baseline-loc.txt

# 4. Snapshot current per-request reqwest sites for H19 progress
rg -n "reqwest::Client::new\(\)" src/ > /tmp/audit-baseline-reqwest.txt
wc -l /tmp/audit-baseline-reqwest.txt     # expect ~50 current production-ish sites, plus any tests/comments

# 5. Confirm DCO is configured
git config --get format.signoff           # should be true; if not: git config format.signoff true
```

**Branching policy** (per CLAUDE.md "Agent / Multi-PR rules"):

- One issue → one branch → one PR.
- Branch name: `fix/audit-{ID}` (e.g. `fix/audit-C7`).
- Always branch from latest `main`. Never fork from another feature branch.
- Max 10 files / 500 lines per PR (excluding `Cargo.lock` and pure docs).
- DCO signed-off for this campaign; **no** `Co-Authored-By`, **no** AI markers unless owner overrides.
- Run `bash scripts/guards/check_pr_scope.sh` and `check_pr_overlap.sh` before push.

**Commit message format** (Conventional Commits, per repo CLAUDE.md):

```
fix(scope): <description>          # bug fix / silent-degradation fix
feat(scope): <description>         # new behavior (e.g. add field)
refactor(scope): <description>     # code-shape change without behavior change
chore(scope): <description>        # tooling, deps

Body explains the *why* (Lore: constraints, rejected alternatives, evidence).
Signed-off-by: <name> <email>
```

---

## 2. Dependency graph

The sequence below is non-negotiable. Items in the same row may go in parallel.

```
                              ┌──────────────────────────────┐
                              │ §3.4 cache helper (blake3)   │──┐
                              │ §3.2 ssrf_guard               │──┼─→ C7, C20
   Wave 0 (Day 1-2)           │ §3.1 default_outbound_client │──┤
                              │ §3.3 PROVIDER_TABLE          │──┘
                              └────────────┬─────────────────┘
                                           │
                                           ↓
   Wave 1                  ┌─ C5  Gemini finish_reason ───┐    (independent, in parallel
   (Day 1-3, can overlap)  ├─ C9  debug-log PII           ├──   with Wave 0)
                           ├─ H11 JWT secret rules        │
                           ├─ H16 delete handlers.rs      │
                           ├─ H18 delete pricing.yaml     │
                           ├─ H22 pricing-source resolver │
                           ├─ M9  audit redactor patterns │
                           ├─ M11 file path validation    │
                           ├─ M16 delete unused ModelPricing
                           ├─ M20 add lite-feature CI lane
                           └─ M24 drop paste rename       ┘
                                           │
                                           ↓
   Wave 2                  ┌─ C7  MCP SSRF (uses §3.2)
   (Day 3-7, P0)           ├─ C8  pgvector SQL identifier
                           ├─ C20 cache key (uses §3.4)
                           ├─ H9  rate-limiter cap + bg cleanup
                           ├─ C13 budget persistence
                           ├─ C14 Redis rate limit (after H9)
                           └─ H19 outbound HTTP migration ─ batched ─→ ~5 sites/PR
                                           │
                                           ↓
   Wave 3                  ┌─ C4  ChatCompletionDelta fields ─→ unblocks C1, C3, H1
   (Week 2-3, P1)          ├─ C19 skip_serializing_if (couples with C4)
                           ├─ C10 core::streaming actix decoupling
                           ├─ C2  Bedrock Tool-role
                           ├─ C5  ✓ done in Wave 1
                           ├─ C6  LiteLLM helper tools
                           ├─ H2  extra_body flatten
                           ├─ H3  seed widening
                           ├─ H7  catalog guard ordering
                           ├─ H8  embeddings/images per provider
                           ├─ H10 CORS expect → fallback
                           └─ H12 audit logger error reporting
                                After C4 lands:
                                ├─ C1  Anthropic streaming
                                ├─ C3  Anthropic non-stream
                                ├─ H1  forward thinking_usage
                                ├─ H4  Anthropic content+tool_calls
                                └─ H5  Anthropic tool_result blocks
                                           │
                                           ↓
   Wave 4                  ┌─ C12 wire pricing migration (independent)
   (Week 4-6, P2)          ├─ C16 AuthConfig::default
                           ├─ C17 dispatch macro from §3.3
                           ├─ C18 single ProviderType source
                           ├─ H17 PydanticAI decision
                           ├─ H20 fail on unresolved ${ENV}
                           ├─ H21 deny_unknown_fields
                           └─ Sequential heavies (one at a time):
                              C11 pricing unification (3 PRs minimum)
                              ↓
                              H15 god-file splits (depends on C11 deletions)
                              ↓
                              C15 team/user convergence (2-step migration)
                              ↓
                              H13 orphan provider dirs (after H8, C18)
                              ↓
                              H14 SDK delegate to UnifiedRouter
                                           │
                                           ↓
   Wave 5                  All M-series (M1–M30 minus already-done)
   (Week 7+, P3)           See §7 below.
```

---

## 3. Wave 0 — Foundations (Day 1–2)

These four cross-cutting helpers from spec §3 must land first because they're consumed by 12+ downstream fixes. Each is a small new module with no behavior change to existing code.

### W0-A. Cache helper module (spec §3.4)

- **Branch**: `chore/audit-cache-helper`
- **Files**:
  - new: `src/core/cache/key_versioning.rs`
  - touched: `src/core/cache/mod.rs` (add `pub mod key_versioning;`)
- **Action**:
  ```rust
  // src/core/cache/key_versioning.rs
  pub const CACHE_KEY_SCHEMA_VERSION: u32 = 2;
  pub fn key_hasher() -> blake3::Hasher { blake3::Hasher::new() }
  pub fn finalize(h: blake3::Hasher) -> String { h.finalize().to_hex().to_string() }
  ```
- **Cargo dep**: `blake3 = "1"` if not already present.
- **Verify**: `cargo check --all-features` plus a unit test that two different inputs yield different hashes.
- **Effort**: XS · 1 hour · no behavior change.
- **Dependency unblocked**: C20.

### W0-B. SSRF guard module (spec §3.2)

- **Branch**: `chore/audit-ssrf-guard`
- **Files**:
  - new: `src/core/net/mod.rs`, `src/core/net/ssrf_guard.rs`
  - touched: `src/lib.rs` (declare `pub mod core::net` if needed)
- **Action**: Copy `is_private_or_reserved_host` and `is_private_or_reserved_ip` from `src/core/a2a/config.rs:250-309` into the new module. Expose `pub fn validate_outbound_url(url: &Url) -> Result<(), SsrfError>`.
- **Migration**: Update `a2a/config.rs` to delegate to the new module. Do NOT touch `mcp/config.rs` yet — that's C7.
- **Verify**: All A2A tests still pass. New `cargo test core::net::ssrf_guard` passes.
- **Effort**: S · 2 hours · regression risk on A2A only, mitigated by tests.
- **Dependency unblocked**: C7.

### W0-C. Outbound HTTP client factory (spec §3.1)

- **Branch**: `chore/audit-outbound-client`
- **Files**:
  - new: `src/core/http/mod.rs`, `src/core/http/outbound.rs`
- **Action**: Implement `default_outbound_client()`, `build_outbound_client(profile)`, `OutboundProfile` per spec §3.1. **Do not migrate any callers in this PR** — that's H19 batched in Wave 2.
- **Verify**: `cargo check --all-features` plus a smoke test that the client builds and times out on a hung server within 5s.
- **Effort**: S · 2 hours · no behavior change to existing callers.
- **Dependency unblocked**: H19, partial C7.

### W0-D. PROVIDER_TABLE skeleton (spec §3.3) — *foundation only*

- **Branch**: `chore/audit-provider-table-skeleton`
- **Files**:
  - new: `src/core/providers/registry/types.rs`
- **Action**: Add `ProviderEntry`, `ProviderKind`, and the static `PROVIDER_TABLE`. Populate ALL existing providers but **do not** generate the enum from it yet — that's C17/C18 in Wave 4. This PR only ships the data table and assertion tests.
- **Tests**: assert that every existing `ProviderType` variant has a matching `PROVIDER_TABLE` row; assert every catalog entry has a row.
- **Verify**: `cargo test core::providers::registry::types`.
- **Effort**: M · 4 hours · no behavior change but needs careful data entry.
- **Dependency unblocked**: C17, C18.

**Wave 0 gate**:
```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features                 # same pass count as kickoff baseline
```

---

## 4. Wave 1 — Quick wins (Day 1–3, can overlap Wave 0)

Ten small, independent fixes. Each is one file or two. Safe to delegate to a Codex/harness agent in parallel, **provided each agent owns disjoint files** (per CLAUDE.md "Agent Isolation" + W-14).

| # | Fix | Files | Duration | Owner candidate |
|---|-----|-------|----------|-----------------|
| W1-1 | C5 Gemini finish_reason → ToolCalls | `src/core/providers/gemini/client.rs` | 30m | self |
| W1-2 | C9 debug-log PII (Bedrock + Milvus + audit similar sites) | `src/core/providers/bedrock/client.rs`, `milvus/provider.rs`, `Cargo.toml` (feature gate) | 1h | self |
| W1-3 | H11 JWT secret hard-rules + example | `src/config/models/auth.rs`, `config/gateway.yaml.example` | 1h | self |
| W1-4 | H16 delete `src/server/handlers.rs` | `src/server/handlers.rs`, `src/server/mod.rs` | 5m | parallel agent |
| W1-5 | H18 delete `config/pricing.yaml` | `config/pricing.yaml` | 5m | parallel agent |
| W1-6 | H22 unify pricing-source resolver | `src/config/models/gateway.rs`, `config/gateway.yaml.example` | 30m | parallel agent |
| W1-7 | M9 audit redactor patterns (Bearer, AKIA, gw-, JWT, sk-ant-) | `src/core/audit/config.rs` | 30m | parallel agent |
| W1-8 | M11 file `path.exists` before validation | `src/core/secret_managers/file.rs` | 10m | parallel agent |
| W1-9 | M16 delete unused `core::providers::ModelPricing` | `src/core/providers/mod.rs` (lines 218-226 + 4 self-tests) | 15m | parallel agent |
| W1-10 | M24 drop `paste = pastey` rename | `Cargo.toml`, all `use paste::` imports | 30m | parallel agent |

**Parallel batch rule**: when delegating, give each agent its file ownership in writing:
```
Agent A owns: src/core/providers/gemini/client.rs only.
Agent B owns: config/pricing.yaml, src/server/handlers.rs only.
...
```

**Wave 1 gate**:
```bash
cargo test --all-features                      # pass count >= baseline
rg "let _ = output\.flush\(\)" src/core/audit  # 0 hits if M9 also did W1-7-adjacent cleanup
```

---

## 5. Wave 2 — P0 critical (Day 3–7)

After Wave 0 helpers exist. Each P0 item gets its own PR. Order within Wave 2 prefers low-dependency items first so reviewers aren't blocked.

### Day 3

#### W2-1. C7 MCP SSRF guard

- **Branch**: `fix/audit-C7-mcp-ssrf`
- **Depends**: W0-B
- **Files**: `src/core/mcp/config.rs` (1 file edit) + new tests
- **Steps**:
  1. Import `crate::core::net::validate_outbound_url`.
  2. In `ServerConfig::validate()`, after the scheme check, call `validate_outbound_url(&parsed)?`.
  3. Add env-var bypass `LITELLM_MCP_ALLOW_PRIVATE_TARGETS=1` with startup `warn!`.
  4. Add tests: 169.254.169.254 rejected, localhost rejected, public host accepted, env-var bypass accepted with warning.
- **Verify**: `cargo test mcp::config::ssrf` + manual curl rejection.
- **Effort**: S · 2 hours.

#### W2-2. C20 cache key blake3 + version

- **Branch**: `fix/audit-C20-cache-key-blake3`
- **Depends**: W0-A
- **Files**: `src/core/cache/key_generator.rs` + new test fixtures
- **Steps**:
  1. Replace every `DefaultHasher::new()` with `key_hasher()`.
  2. Prepend `CACHE_KEY_SCHEMA_VERSION.to_le_bytes()` to every key domain.
  3. Hash full normalized JSON of `tools`, `tool_choice`, `response_format` (incl. `json_schema`), `parallel_tool_calls`, `reasoning_effort`, `service_tier`, `logit_bias`.
  4. Hash `tool_calls`, `tool_call_id`, `function_call` on assistant messages.
  5. Add tests: same request → same key; different `tool.parameters` → different keys; bumping `CACHE_KEY_SCHEMA_VERSION` shifts all keys.
- **Verify**: `cargo test cache::key_generator`.
- **Effort**: S · 3 hours.
- **Migration note**: Rolling out invalidates existing Redis cache once. Acceptable cold-start cost.

#### W2-3. C9 debug-log PII (if not done in Wave 1)

If parallelized in Wave 1: skip.
If still pending: see W1-2.

### Day 4

#### W2-4. H9 rate-limiter cap + background cleanup

- **Branch**: `fix/audit-H9-ratelimit-cap`
- **Depends**: none
- **Files**: `src/server/middleware/auth_rate_limiter.rs`, `src/server/http.rs` (start bg task)
- **Steps**:
  1. Add `max_entries: usize` to `AuthRateLimiterConfig` (default 100k).
  2. On insert, if over cap, evict oldest entries (LRU via timestamp).
  3. Spawn a tokio task in `HttpServer::new` that runs `cleanup_old_entries()` every 60s.
  4. Remove the probabilistic cleanup in auth handlers (now redundant).
- **Verify**: integration test inserting 200k entries → map size stays at 100k.
- **Effort**: S · 3 hours.

#### W2-5. C8 pgvector SQL identifier

- **Branch**: `fix/audit-C8-pgvector-sql`
- **Files**: `src/core/providers/pg_vector/{config.rs,provider.rs}`
- **Steps**:
  1. In `PgVectorConfig::validate()`, reject schema/table_name not matching `^[A-Za-z_][A-Za-z0-9_]{0,62}$`.
  2. Replace `format!(" LIMIT {}", options.limit)` with bind parameter.
  3. Replace `to_sql_string` with driver's parameterized API; delete the hand-rolled `'` escape.
  4. Validate `operator` against allowed set.
- **Verify**: unit test with malicious table name returns validation error; integration test against real Postgres confirms no SQL injection.
- **Effort**: M · 4 hours.

### Day 5–6

#### W2-6. C13 budget persistence

- **Branches** (split into 2 PRs):
  - `fix/audit-C13a-budget-schema` — new migration + entity
  - `fix/audit-C13b-budget-restore` — `save`/`restore` + startup wiring
- **Files**:
  - new: `src/storage/database/migration/m20260501_000001_create_budget_spend_table.rs`
  - new: `src/storage/database/entities/budget_spend.rs`
  - touched: `src/core/budget/provider_limits.rs`, `src/server/state.rs`, `migration/mod.rs`, `entities/mod.rs`
- **Steps**:
  1. PR-a: ship migration + entity + register in `mod.rs`. No behavior change yet.
  2. PR-b: implement `save(&DatabaseStore)` and `restore(&DatabaseStore)`. Call `restore()` in `AppState::new_with_unified_router` BEFORE serving traffic. Add periodic flush every 30s + on graceful shutdown.
- **Verify**: integration test "spend $5, restart, assert $5".
- **Effort**: M · 6 hours total.
- **Risk**: if migration fails on existing DBs with un-migrated data, restore must fall back gracefully (return zero spend with `error!`). Document the eventual-consistency window of ≤30s in PR description.

#### W2-7. C14 Redis-backed rate limit

- **Branch**: `fix/audit-C14-redis-ratelimit`
- **Depends**: H9 (W2-4) for the in-process side
- **Files**: `src/core/rate_limiter/`, `src/server/state.rs`, `src/config/models/`
- **Steps**:
  1. Define a `RateLimiter` trait with `pub async fn allow(...) -> bool`.
  2. Move existing DashMap impl behind `InProcessRateLimiter`.
  3. Add `RedisRateLimiter` using a Lua token-bucket script.
  4. Selection at `AppState::new` based on `redis.enabled`.
  5. **Fail-closed for HA**: if `redis.enabled = false` and replicas > 1 (env `LITELLM_REPLICAS` or autodetect), refuse to start with `enable_rate_limit: true` — emit `error!` and exit.
- **Verify**: multi-replica integration test; single-replica existing tests still green.
- **Effort**: M · 6 hours.

### Day 7

#### W2-8. H19 outbound HTTP migration — first batch

- **Branch**: `refactor/audit-H19-outbound-batch1`
- **Depends**: W0-C
- **Files**: 5–8 sites/PR. Batch order starts with the raw report sites below, then continue until `rg "reqwest::Client::new\(\)" src/ -g '!**/*test*' -g '!**/tests.rs'` reaches only intentional exceptions:
  - **Batch 1** (Day 7): `services/pricing/service.rs:40`, `monitoring/alerts/channels.rs:103`, `core/observability/metrics.rs:89,103`, `core/observability/logging.rs:105`
  - **Batch 2** (Day 8): `core/providers/codestral/provider.rs:273`, `core/providers/databricks/provider.rs:438,481`, `core/providers/baseten/provider.rs:251`, `core/providers/deepgram/provider.rs:228,324`
  - **Batch 3** (Day 9): `core/providers/github_copilot/provider.rs:407,474,535`, `core/providers/oci/provider.rs:392`, `core/providers/azure_ai/image_generation.rs:27`, `core/providers/gradient_ai/provider.rs:283`
  - **Batch 4** (Day 10): `core/providers/vertex_ai/auth.rs:145`, `core/providers/exa_ai/provider.rs:181`, `core/providers/firecrawl/provider.rs:107`, `core/providers/v0/mod.rs:176`, `core/providers/azure/client.rs:25`, `storage/vector/qdrant.rs:21`, `core/a2a/provider.rs:302`
  - **Follow-up batches**: cover remaining providers found by the baseline snapshot, including rerank, OpenAI API methods/client, pg_vector, watsonx, GitHub, Amazon Nova, Ollama, Replicate, Clarifai, Stability, ElevenLabs, Snowflake, AI21, Empower, and Datarobot.
- **Action per site**: replace `reqwest::Client::new()` with `default_outbound_client().clone()`.
- **Verify per batch**: existing tests still pass; site count drops by the batch size in `rg "reqwest::Client::new\(\)" src/`.
- **Effort**: 4× M · 1 hour each.

**Wave 2 gate**:
```bash
cargo test --all-features
rg "DefaultHasher" src/core/cache/        # 0 hits
rg "reqwest::Client::new\(\)" src/ | wc -l # ≤ 3 (only inside core/http/outbound.rs)
# Manual: replay a budget+restart test, confirm spend persists.
```

---

## 6. Wave 3 — P1 provider correctness (Week 2–3)

The keystone is **C4** (`ChatCompletionDelta` field expansion). Land it first; it unblocks C1, C3, H1.

### Week 2, Day 1

#### W3-A. C4 + C19 — `ChatCompletionDelta` and `skip_serializing_if`

- **Branch**: `fix/audit-C4-streaming-delta-fields`
- **Files**: `src/core/streaming/types.rs`, `src/server/routes/ai/chat.rs:553-560`
- **Steps**:
  1. Extend `ChatCompletionDelta` with `thinking`, `thinking_signature`, `refusal`, `tool_call_id`, `function_call`, all with `#[serde(skip_serializing_if = "Option::is_none")]`.
  2. Apply `skip_serializing_if` to every Option in `ChatCompletionChunk`, `ChatCompletionChunkChoice` (covers C19).
  3. Update `convert_core_chunk_to_streaming` to forward the new fields.
- **Verify**: serialization test asserts no `null` fields; round-trip with `ChatDelta { thinking: Some(...) }` survives.
- **Effort**: S · 3 hours.
- **Unblocks**: C1, C3, H1.

### Week 2, Day 2–4 (parallel-safe after C4 lands)

| # | Fix | Files | Effort |
|---|-----|-------|--------|
| W3-1 | C1 Anthropic streaming tool/thinking deltas | `src/core/providers/base/sse.rs`, `src/core/types/responses/delta.rs` | M · 6h |
| W3-2 | C3 Anthropic non-stream thinking + cache usage | `src/core/providers/anthropic/client.rs`, `src/core/types/responses/usage.rs` | S · 4h |
| W3-3 | H1 forward `thinking_usage` in `convert_usage` | `src/server/routes/ai/chat.rs` | XS · 1h |
| W3-4 | C2 Bedrock Tool-role / ToolResult | `src/core/providers/bedrock/chat/converse.rs` | M · 5h |
| W3-5 | C6 LiteLLM helper drops tools | `src/core/completion/{conversion.rs,types.rs}` | S · 3h |
| W3-6 | H2 expose `parallel_tool_calls`/`extra_body` flatten (also fixes M2) | `src/core/models/openai/requests.rs`, `chat.rs` | S · 3h |
| W3-7 | H3 seed widening | `src/core/types/chat.rs`, `chat.rs` | XS · 30m |
| W3-8 | H4 Anthropic content + tool_calls coexist | `src/core/providers/anthropic/client.rs:480-498` | XS · 1h |
| W3-9 | H5 Anthropic tool_result blocks | `src/core/providers/anthropic/client.rs:399-401` | S · 2h |
| W3-10 | H7 catalog guard ordering | `src/core/providers/factory/registry.rs` | XS · 1h |
| W3-11 | H8 embeddings/images per provider | `src/core/providers/mod.rs:481-512` + each provider's `Provider` trait impl | M · 6h |
| W3-12 | H10 CORS expect → graceful | `src/server/http.rs:113` | XS · 1h |
| W3-13 | H12 audit logger error reporting | `src/core/audit/logger.rs:131-132,166-169` | XS · 1h |
| W3-14 | C10 `core::streaming` actix decoupling | `src/core/streaming/mod.rs`, new `src/server/sse.rs` | S · 3h |
| W3-15 | M5 Anthropic stop_sequence (rolled into C3) | covered by C3 | — |
| W3-16 | M6 Gemini cache + thoughts tokens | `src/core/providers/gemini/client.rs:514-530` | S · 2h |

**Sequencing note**: H4, H5, H8, M6 all touch same provider directories. Avoid concurrent edits — single owner, serial PRs per provider.

**Wave 3 gate**:
```bash
cargo test --all-features
# Integration tests for the new fixtures:
cargo test anthropic_stream_tool_use
cargo test anthropic_thinking_roundtrip
cargo test bedrock_tool_result_round_trip
cargo test gemini_tool_call_finish_reason
rg "actix_web" src/core | wc -l           # 0
```

---

## 7. Wave 4 — P2 architectural consolidation (Week 4–6)

Heavy refactors. Mix of independent quick fixes (parallel) and sequential heavies (one at a time).

### Week 4 — Parallel quick fixes (Day 1–3)

| # | Fix | Files | Effort |
|---|-----|-------|--------|
| W4-1 | C12 wire pricing migration + entity | `src/storage/database/migration/{mod.rs,m20240201...}`, `entities/{mod.rs,pricing.rs,pricing_history.rs}` | S · 3h |
| W4-2 | C16 `AuthConfig::default` valid | `src/config/models/auth.rs:51-63` | XS · 30m |
| W4-3 | H17 PydanticAI: implement or remove | `src/core/providers/provider_type.rs:36`, factory test | S · 2h |
| W4-4 | H20 fail on unresolved `${ENV}` | `src/config/mod.rs:32-57`, `validation/config_validators.rs` | S · 3h |
| W4-5 | H21 `deny_unknown_fields` | all top-level config structs (~12 files) | M · 6h |
| W4-6 | M14 hot-reload watcher (or downgrade comment) | `src/server/state.rs:21-26` | M · 4h |

### Week 4–5 — Sequential heavy: C11 Pricing unification

The largest single refactor in P2. Split into 3 PRs to stay under the 500-line/PR rule.

#### W4-C11a. Migrate `core::cost::CostCalculator` callers to `PricingService`

- **Branch**: `refactor/audit-C11a-pricing-callers`
- **Files**: every caller of `core::cost::CostCalculator::calculate_*` (likely ~10 sites in `core/router`, `core/observability`, etc.)
- **Action**: replace direct calls with `app_state.pricing_service.calculate(...)`. Keep the old API as a `#[deprecated]` shim until W4-C11b.
- **Verify**: `cargo test --all-features`; cost values unchanged in regression tests.
- **Effort**: M · 6h.

#### W4-C11b. Delete `core::providers::base::pricing` and per-provider pricing

- **Branch**: `refactor/audit-C11b-delete-base-pricing`
- **Files**: delete `src/core/providers/base/pricing.rs`; remove `Provider::calculate_cost` direct DB use; remove `anthropic::ModelPricing` re-exports.
- **Action**: `Provider::calculate_cost` now accepts `&PricingService` (extend trait method). Update all impls.
- **Verify**: `rg "ModelPricing\b" src/` returns exactly 1 struct definition (in `services::pricing`).
- **Effort**: L · 8h (touches every provider).
- **Risk**: large blast radius. Submit as a single coherent PR; merge during a low-traffic window if any.

#### W4-C11c. Delete `core::cost::calculator.rs` god file

- **Branch**: `refactor/audit-C11c-delete-cost-calculator`
- **Files**: delete `src/core/cost/calculator.rs` (1645 lines), keep `src/core/cost/mod.rs` re-exporting from `services::pricing`.
- **Action**: per CLAUDE.md "No backward compatibility — break old formats freely".
- **Verify**: full test suite green.
- **Effort**: M · 4h.

### Week 5 — H15 god-file splits

After C11 deletes ~half of `cost/calculator.rs`, split the remaining files. Each split is its own PR.

| # | File (current LOC) | Target split | Effort |
|---|---|---|---|
| W4-G1 | `core/providers/anthropic/models.rs` (1268) | `models/{registry,features,pricing}.rs` (pricing already deleted by C11) | M · 4h |
| W4-G2 | `core/providers/base/sse.rs` (1251) | `base/sse/{parser,event,transformers/{anthropic,openai,gemini,cohere,databricks}}.rs` | L · 8h |
| W4-G3 | `core/providers/anthropic/client.rs` (1152) | `client/{http,retry,error_mapping,tool_call}.rs` | M · 6h |
| W4-G4 | `server/routes/ai/chat.rs` (699) | `chat/{request_pipeline,response_pipeline,streaming_dispatch}.rs` | M · 4h |
| W4-G5 | `core/providers/openai_like/provider.rs` (769) | `provider.rs` core + `request.rs` + `headers.rs` | S · 3h |

### Week 5–6 — Sequential heavy: C15 Team/User convergence

Two-step migration. Step 2 is destructive; requires four-point confirmation per `vibeguard W-10`.

#### W4-C15a. Copy `um_*` data to canonical tables

- **Branch**: `fix/audit-C15a-migrate-um-data`
- **Files**: new migration `m20260501_000002_migrate_um_to_canonical.rs`; update `user_management_ops.rs` to read from `users`/`teams` (write to both during transition).
- **Verify**: integration test creates user via `um_*` flow, queries via canonical flow, asserts visible.
- **Effort**: L · 8h.

#### W4-C15b. Drop `um_*` tables

- **Branch**: `fix/audit-C15b-drop-um-tables`
- **Files**: new migration `m20260501_000003_drop_um_tables.rs`; delete `seaorm_db/user_management_ops.rs`.
- **Pre-merge gate** (per W-10):
  ```
  --- Publish Confirmation ---
  Target: drop tables um_users, um_teams, um_organizations
  Scope: removes legacy user-management raw-SQL track
  Untouched: users, teams, organizations tables (already canonical after W4-C15a)
  Command: cargo run -- database migrate
  ---
  Approved? [y/n]
  ```
- **Verify**: `rg "um_users|um_teams" src/` returns 0.
- **Effort**: M · 4h.

### Week 6 — H13, C17, C18, H14

#### W4-C17/C18. ProviderType single source of truth

- **Branch**: `refactor/audit-C17-C18-provider-table-codegen`
- **Depends**: W0-D
- **Files**: `src/core/providers/{mod.rs,provider_type.rs,factory/registry.rs}`
- **Action**: generate `ProviderType` enum, `From<&str>`, `FromStr`, `Display`, `factory_supported_provider_types()` from `PROVIDER_TABLE`. Replace 4-arm `dispatch_provider!` macro body with one driven by the table.
- **Verify**: `cargo expand` snapshot of `dispatch_provider!` matches a checked-in expected output. Every alias resolves.
- **Effort**: L · 10h.

#### W4-H13. Orphan provider dirs

- **Branch**: `chore/audit-H13-orphan-providers`
- **Depends**: H8 (per-provider trait impls), C18 (single source of truth)
- **Action**: for each of the 35+ provider directories under `providers-extended` feature, decide:
  - **Keep + wire**: add to `PROVIDER_TABLE`, implement factory branch, write a smoke test.
  - **Delete**: `git rm -r src/core/providers/<name>/`, remove from `mod.rs`.
- **Process**: one commit per directory keep-or-delete decision (separate PRs grouped by 5 providers each to stay under scope rule).
- **Effort**: L · 12h spread over Week 6.
- **Pre-merge gate** (each delete batch): four-point confirmation listing exactly which directories are deleted.

#### W4-H14. SDK delegate to UnifiedRouter

- **Branch**: `refactor/audit-H14-sdk-router-delegate`
- **Files**: `src/sdk/client/{routing.rs,llm_client.rs,stats.rs}`
- **Action**: replace `LoadBalancer` with a thin wrapper around `core::router::UnifiedRouter`. Delete `sdk::client::routing.rs` if fully covered.
- **Verify**: assertion test that `LLMClient::send` and `completion()` produce the same provider selection given the same seed.
- **Effort**: M · 6h.

**Wave 4 gate**:
```bash
cargo test --all-features
cargo check --no-default-features --features lite     # P2 must make this pass
rg "ModelPricing\b" src/                              # exactly 1 struct definition
rg "um_users|um_teams" src/                           # 0
wc -l src/core/cost/calculator.rs 2>/dev/null         # file should not exist
ls src/core/providers/<orphan-name>/ 2>/dev/null      # for each H13 deletion: not exist
```

---

## 8. Wave 5 — Hygiene (Week 7+)

All M-series not yet covered. Most are 5–30 minute fixes; can be batched into 2–3 sweep PRs.

### Sweep PR group A: trivial deletions / single-line fixes

- M1 `chat.rs:562-564` — replace `.ok()` with `unwrap_or_else + error!`
- M11 ✓ (Wave 1)
- M16 ✓ (Wave 1)
- M18 `OpenAILikeProvider::name` — change trait to `&str` or `Cow<'static, str>`
- M22 `Cargo.toml:195` — re-add or rename `vector-db` feature
- M24 ✓ (Wave 1)
- M28 delete `core/streaming/providers.rs` (after Wave 3 confirms `base/sse.rs` covers everything)

### Sweep PR group B: redactor / observability

- M9 ✓ (Wave 1)
- M10 forgot-password constant-time
- M12 `openai_like/provider.rs:406` — generic message + correlation id
- M15 redis-enabled-but-unused warning at startup

### Sweep PR group C: SSRF hardening

- M7 A2A DNS rebinding — custom `dns_resolver` on `ClientBuilder`
- M8 MCP tool-description hash baseline (SEC-12)

### Sweep PR group D: feature-flag rationalization

- M20 ✓ (Wave 1)
- M21 split `metrics-system` from `metrics-request`
- M22 ✓ (above)
- M25 curated `prelude` mod in `lib.rs`

### Sweep PR group E: env-var consolidation

- M13 introduce `LITELLM_HOME`; derive sub-paths from it

### Sweep PR group F: macros + leftover cleanup

- M19 rename `mcp::AuthType` → `McpAuthType`
- M23 audit and prune `core::providers::macros/`

---

## 9. Daily rhythm template

For each working day during the campaign:

```
1. Pull main:          git pull --ff-only origin main
2. Check progress:     grep -c "^| .*| .*DONE |" PLAN_AUDIT_REMEDIATION.md
3. Pick next item:     find first row in §8 of PLAN_AUDIT_REMEDIATION.md with Status=TODO
                       and Phase matching current Wave
4. Branch:             git checkout -b fix/audit-{ID}
5. Implement per spec section in PLAN_AUDIT_REMEDIATION.md
6. Verify locally:
                       cargo fmt --all -- --check
                       cargo clippy --all-targets --all-features -- -D warnings
                       cargo test --all-features
                       bash scripts/guards/check_pr_scope.sh
                       bash scripts/guards/check_pr_overlap.sh
7. Commit (DCO signed, no AI markers, conventional format)
8. Push + open PR with the spec's verification commands pasted as evidence
9. Update PLAN_AUDIT_REMEDIATION.md row: PR=#NNN, Status=IN_PROGRESS or DONE
10. End of day: short note in PLAN_AUDIT_PROGRESS.md (optional, see §11)
```

---

## 10. Risk register and rollback playbook

| Risk | Likelihood | Impact | Mitigation | Rollback |
|------|------------|--------|------------|----------|
| C13 budget restore loses data on schema mismatch | Medium | Data loss on first deploy | Migration ships in PR-a (no behavior change); restore added in PR-b reads-or-zeroes with `error!` on schema mismatch | Revert PR-b; in-memory state still works (current behavior) |
| C14 Redis Lua bucket race on multi-replica | Low | Quota over-allowance | Lua script tested with 2-replica integration test | Switch back to `InProcessRateLimiter` via config; restart |
| C15b dropping `um_*` tables loses data | High if step 2 not done | Production data loss | Two-step migration (copy first, drop later); manual confirmation gate | Restore from pre-merge DB snapshot; un-drop migration is the new `m20260501_000004_restore_um_tables` |
| C11b `Provider::calculate_cost` API change breaks SDK consumers | Medium | External consumer break | Acknowledge in 0.x SemVer; document in CHANGELOG | Revert PR; old API returns |
| H13 deleting orphan provider dir that someone uses externally | Low | External break | Each delete batch gets 4-point confirmation; deletes go in their own commits for easy revert | `git revert` the specific batch |
| C17/C18 macro codegen drift | Low | Provider variant invisible | `cargo expand` snapshot test catches drift on every PR | Revert; the test ensures no silent diff |
| §3.4 cache key version bump invalidates Redis cache | Cert | Cold-cache window of one TTL | Acceptable; document in P0 release notes | None needed (cache regenerates) |
| C7 SSRF guard rejects legitimate localhost MCP usage | Medium | Local dev breaks | Env-var bypass `LITELLM_MCP_ALLOW_PRIVATE_TARGETS=1` documented in CONTRIBUTING.md | Set env var |

**General rollback rule**: each fix is one PR. `git revert <merge-commit>` always works. No squashes that hide individual fixes (per CLAUDE.md commits-are-atomic).

---

## 11. Optional: progress journal

Maintain `PLAN_AUDIT_PROGRESS.md` (gitignored or root-level — your choice) with one section per closed week:

```
## Week N (YYYY-MM-DD to YYYY-MM-DD)
Closed: C5, C9, H11, M9, M11, M16, M24 (7 items)
Open:   W0-A, W0-B (in PR review)
Blocked: C13b waiting on schema decision (Q2)
Next:   C7, C20 (ready once W0-A/B merge)
Notes:  Found unexpected dep on `core::cache` from `litellm-api`; opened S-1 follow-up.
```

This is not required by any rule — it's an aid for hand-off. Skip if the tracker in `PLAN_AUDIT_REMEDIATION.md` §8 is enough.

---

## 12. Done criteria

The campaign closes when:

1. Every row in `PLAN_AUDIT_REMEDIATION.md` §8 has `Status=DONE` or a documented decision (e.g., `H6` may stay `DEFER` if owner confirms).
2. All Wave gate commands in §3–§7 are green on `main`.
3. The full "Verification matrix" in `PLAN_AUDIT_REMEDIATION.md` §7 passes.
4. CHANGELOG.md has entries summarizing the campaign across the affected versions.
5. A follow-up audit (re-run `codebase-audit` skill) shows the original findings are gone and no new Critical/High items appear.

After (5), this plan and the spec can be archived to `docs/audit-2026-05-01/`.
