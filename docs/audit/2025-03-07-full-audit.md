# Full Project Audit Report — 2025-03-07

Performed by 4 parallel agent roles: Architect, Provider Engineer, Router & Performance, Security & Auth.

> **Update 2025-03-07**: P0 issues #1-#3 and P1 issues #4-#7 have been fixed and verified. See [Fix Log](#fix-log) at the end.

---

## Project Overview

| Metric | Value |
|--------|-------|
| Language | Rust, ~330K lines |
| Version | 0.4.2 |
| Providers | 87+ AI providers |
| Architecture | Single crate, async-first, trait-based |

---

## 1. Architecture (Architect)

### Critical Issues

| Severity | Issue | Details |
|----------|-------|---------|
| HIGH | `core/` is a God Module | 30+ submodules mixing providers, protocols (MCP/A2A), security, observability |
| HIGH | `providers/mod.rs` — 2352 lines | Provider enum + 5 dispatch macros + ProviderType + factory methods. All provider changes converge here |
| HIGH | Middleware system is dead code | `MiddlewareStack.execute_chain()` always returns `"Middleware system temporarily disabled"` |
| HIGH | 55+ files exceed 800 lines | Worst: `providers/mod.rs` at 2352 lines (U-16 violation) |
| MEDIUM | Duplicate `HealthStatus` | Defined in both `core/models/mod.rs:211` and `core/types/health.rs:10` with different trait impls |
| MEDIUM | Transformer system is a stub | `TransformerRegistry.register_transformer()` is a no-op; contains unsafe pin operation |
| LOW | Stale comments in `core/mod.rs` | Comments claim `virtual_keys` is removed, but it exists with active code |

### Recommended Split

`core/` should eventually be split into: `provider-core`, `router`, `protocols`, `security`, `observability`, `platform-services`.

### Files >800 Lines (Top 20)

| File | Lines |
|------|-------|
| `core/providers/mod.rs` | 2352 |
| `utils/error/utils.rs` | 1397 |
| `core/providers/vertex_ai/client.rs` | 1440 |
| `core/providers/gemini/models.rs` | 1364 |
| `core/providers/openai/transformer.rs` | 1343 |
| `core/providers/azure/assistants.rs` | 1257 |
| `core/budget/types.rs` | 1242 |
| `core/cost/calculator.rs` | 1230 |
| `utils/error/gateway_error/conversions.rs` | 1229 |
| `core/providers/azure/batches/mod.rs` | 1194 |
| `core/providers/jina/mod.rs` | 1184 |
| `core/router/strategy_impl.rs` | 1183 |
| `core/providers/base/sse.rs` | 1175 |
| `sdk/types.rs` | 1163 |
| `core/providers/vertex_ai/embeddings/mod.rs` | 1139 |
| `core/providers/bedrock/model_config.rs` | 1097 |
| `core/providers/bedrock/utils/cost.rs` | 1088 |
| `core/providers/azure/responses/transformation.rs` | 1070 |
| `core/analytics/types.rs` | 1071 |
| `core/providers/anthropic/client.rs` | 1054 |

---

## 2. Provider System (Provider Engineer)

### Issues

| Priority | Issue | Location |
|----------|-------|----------|
| P1 | ProviderType enum and Catalog dual-track redundancy | `mod.rs:258-333` — Groq/XAI/DeepSeek etc. already in catalog but enum variants remain as dead code |
| P1 | Streaming path bypasses connection pool | `openai_like/provider.rs:145` — creates temporary `reqwest::Client::new()` instead of reusing `pool_manager` |
| P1 | Bearer Header repeated in 42 files | 95 occurrences of header construction; streaming path doesn't reuse `get_request_headers()` |
| P2 | Gemini SSE usage parsing duplicated | `base/sse.rs:636-743` — identical Usage construction logic appears twice |
| P2 | Error category inconsistency | `openai_like/error.rs:57` — `ApiError` returns `"other"` not `"api_error"` |
| P2 | RateLimit retry_after hardcoded to 60s | `provider_error_conversions.rs:27` — loses original retry-after info |
| P2 | health_check() bypasses pool_manager | `openai_like/provider.rs:416-430` — creates temporary client |

### Verified Good

- Tier 1 Catalog mechanism (53 data-driven providers)
- Unified `ProviderError` type design
- `UnifiedSSEParser<T>` + `SSETransformer` trait architecture
- `dispatch_provider_async!` macro (justified: >5 repetitions)
- `factory_supported_provider_types()` guard prevents silent degradation

---

## 3. Router & Performance

### Issues

| Priority | Issue | Location |
|----------|-------|----------|
| P1 | **Rate limit middleware is a shell** | `server/middleware/rate_limit.rs:79-86` — only debug log, no actual limiting |
| P1 | Rate limiter global write lock hotspot | `rate_limiter/strategies.rs` — all three strategies hold write lock for entire computation |
| P1 | Memory cache TOCTOU race | `cache/memory.rs:81,92` — check-then-act gap in get/set |
| P2 | LRU update is O(n) linear scan | `cache/memory.rs:246` — `VecDeque::iter().position()` on every cache hit |
| P2 | `update_stats` full scan on every write | `cache/memory.rs:383` — sums all entries on every set/delete |
| P2 | DashMap nested shard lock contention | `router/selection.rs:61-83` — filter closure holds two DashMap shard locks |
| P3 | EMA update non-atomic | `router/deployment.rs:323-351` — load/store gap, acceptable for approximate metrics |

### Verified Good

- 7 routing strategies with clean `RoutingContext` snapshot pattern
- `DeploymentState` fully atomic (AtomicU32/U64/U8, Relaxed ordering)
- `check_and_record` atomic operation (deprecated separate `record()`)
- `AuthRateLimiter` exponential backoff
- L1 memory + L2 Redis two-tier cache architecture
- Clean `execute` -> `execute_with_retry` -> `select_deployment` -> `release_deployment` chain

---

## 4. Security Audit

### Issues

| Severity | Category | Location | Issue |
|----------|----------|----------|-------|
| HIGH | SQL Injection | `pg_vector/provider.rs:207` | `full_table_name()` concatenated into SQL without identifier quoting |
| HIGH | SSRF | `a2a/provider.rs:83` | A2A URL accepts any target including internal IPs (169.254.x, localhost) |
| HIGH | Missing Rate Limiting | `rate_limit.rs:79-84` | AI inference endpoints have no rate protection — cost attack risk |
| MEDIUM | Info Leakage | `routes/auth/password.rs:17` | Password reset logs full email address |
| MEDIUM | Weak Default Key | `gateway.yaml.example:98` | Example JWT secret is a usable string, not caught by validate() |
| MEDIUM | Unauthenticated /metrics | `middleware/helpers.rs:50` | Exposes system architecture info without auth |
| MEDIUM | JWT HS256 hardcoded | `jwt/handler.rs:19` | No support for RS256/ES256 algorithm selection |
| MEDIUM | Auth token parsing ambiguity | `middleware/helpers.rs:11-19` | `gw-` prefix treated as API key, could conflict with JWT |
| LOW | unsafe env::set_var | `secret_managers/env.rs:75` | In production path, not gated by `#[cfg(test)]` |
| LOW | Password reset incomplete | `auth/password.rs:66-69` | Token generated but email never sent (TODO comment) |

### Verified Secure

- API keys stored as hashes only, raw key returned once on creation
- Passwords use argon2 hashing
- JWT Debug output redacts secrets `[REDACTED]`
- Request context filters out Authorization headers
- MCP defaults to deny_all policy
- IP access control: allowlist/blocklist/CIDR/IPv6 complete
- Password reset: anti-enumeration (same response regardless of email existence)
- Security headers: HSTS, X-Frame-Options, X-Content-Type-Options, X-XSS-Protection
- Guardrails cover prompt injection and PII patterns (Block/Mask/Log modes)
- CORS: conservative defaults, no wildcard `*`

---

## 5. Priority Matrix

### P0 — Fix Immediately

| # | Issue | Risk | Status |
|---|-------|------|--------|
| 1 | SQL injection in pg_vector table name | Data breach | FIXED |
| 2 | SSRF in A2A URL validation | Internal network exposure | FIXED |
| 3 | Rate limit middleware is a shell | Cost attack / DoS | FIXED |

### P1 — Fix This Sprint

| # | Issue | Impact | Status |
|---|-------|--------|--------|
| 4 | Rate limiter global write lock | Performance bottleneck at scale | FIXED |
| 5 | Streaming path bypasses connection pool | Resource waste, connection exhaustion | FIXED |
| 6 | Memory cache TOCTOU race | Data inconsistency | FIXED |
| 7 | Log email in password reset | Privacy / compliance | FIXED |

### P2 — Fix This Month

| # | Issue | Impact |
|---|-------|--------|
| 8 | ProviderType enum dead code cleanup | Maintenance burden |
| 9 | LRU O(n) -> O(1) | Cache performance |
| 10 | /metrics endpoint auth | Info disclosure |
| 11 | JWT algorithm configurability | Security flexibility |
| 12 | Bearer header deduplication | Code quality |
| 13 | Error category inconsistency | Debugging accuracy |

### P3 — Medium Term

| # | Issue | Impact |
|---|-------|--------|
| 14 | `providers/mod.rs` split (<800 lines) | Maintainability |
| 15 | 55+ oversized files treatment | Code quality |
| 16 | Dead code cleanup (Middleware/Transformer) | Clarity |
| 17 | HealthStatus deduplication | Type safety |
| 18 | `core/` workspace crate split | Build times, modularity |

---

## Fix Log

### P0-1: SQL Injection in pg_vector (FIXED)

**Files modified**:
- `src/core/providers/pg_vector/config.rs`
- `src/core/providers/pg_vector/provider.rs`

**Changes**:
1. `full_table_name()` now wraps schema and table_name with PostgreSQL identifier double quotes: `"schema"."table_name"`
2. `validate()` added `[a-zA-Z0-9_]` allowlist for both `table_name` and `schema` fields
3. `create_index_sql()` index name wrapped with double quotes
4. `stats_sql()` fixed `::regclass` cast to use identifier quoting instead of single quotes
5. 3 new security tests added, 39 total pg_vector tests pass

### P0-2: SSRF in A2A URL Validation (FIXED)

**Files modified**:
- `src/core/a2a/config.rs`

**Changes**:
1. Added `extract_url_host()` — parses host from URL, handles IPv6 bracket notation and ports
2. Added `is_private_or_reserved_host()` — checks known internal hostnames (localhost, metadata.google.internal, etc.)
3. Added `is_private_or_reserved_ip()` — blocks RFC 1918 (10.x, 172.16.x, 192.168.x), loopback (127.x, ::1), link-local (169.254.x), unspecified (0.0.0.0), IPv6 unique-local (fc00::/7), and IPv4-mapped IPv6 addresses
4. `validate()` now rejects URLs targeting private/reserved addresses
5. 17 new SSRF tests added, 28 total a2a tests pass
6. No new external dependencies — uses only `std::net`

**Known limitation**: DNS rebinding attacks (public domain resolving to private IP) require runtime TCP connection target verification, deferred as separate item.

### P0-3: Rate Limit Middleware Shell (FIXED)

**Files modified**:
- `src/server/middleware/rate_limit.rs`

**Changes**:
1. `_requests_per_minute` renamed back to `requests_per_minute` and used for actual limiting
2. Added `extract_client_key()` — extracts client identity from Authorization header > X-Forwarded-For > peer IP
3. Added `RateLimitError` implementing `ResponseError` — returns HTTP 429 with `Retry-After` header
4. Added `KeyTracker` struct with sliding window counter as fallback
5. Dual-path rate limiting in `call()`:
   - **Primary**: calls `get_global_rate_limiter().check_and_record()` for atomic rate checking
   - **Fallback**: when global limiter unavailable, uses in-process `DashMap<String, KeyTracker>` sliding window
6. 232 existing tests pass, zero regressions

---

### P1-4: Rate Limiter Global Write Lock (FIXED)

**Files modified**:
- `src/core/rate_limiter/limiter.rs`
- `src/core/rate_limiter/strategies.rs`
- `src/core/rate_limiter/utils.rs`

**Changes**:
1. Replaced `Arc<RwLock<HashMap<String, RateLimitEntry>>>` with `Arc<DashMap<String, RateLimitEntry>>` for per-key lock granularity
2. All three strategies (SlidingWindow, TokenBucket, FixedWindow) now use `DashMap::entry()` API instead of global write lock
3. Expired entry cleanup is now lazy (per-key on access) plus background `DashMap::retain` (per-shard, not global)
4. No new dependencies — `dashmap` already in Cargo.toml
5. 64 rate limiter tests pass

### P1-5: Streaming Path Bypasses Connection Pool (FIXED)

**Files modified**:
- `src/core/providers/openai_like/provider.rs`

**Changes**:
1. `execute_chat_completion_stream`: replaced `streaming_client()` + manual header construction with `self.pool_manager.client()` + `get_request_headers()` + `apply_headers()`
2. `health_check()`: replaced `reqwest::Client::new()` with `self.pool_manager.client()`
3. Fixed bug: old streaming code skipped `self.config.base.headers` — now included via `get_request_headers()`
4. 7 existing unit tests pass

### P1-6: Memory Cache TOCTOU Race (FIXED)

**Files modified**:
- `src/core/cache/memory.rs`
- `src/core/cache/types.rs`

**Changes**:
1. TOCTOU in `get()`/`exists()`: replaced two-step get+remove with DashMap's `remove_if()` for atomic conditional delete
2. TOCTOU in `set_with_ttl()`: replaced `contains_key()` + `insert()` with `DashMap::insert()` return value check
3. LRU O(n) → O(1): replaced `VecDeque` + `iter().position()` with `lru::LruCache` (already in Cargo.toml)
4. Eliminated `update_stats()` full scan: added `add_total_size()`/`sub_total_size()` to `AtomicCacheStats` for O(1) incremental tracking
5. 132 cache tests pass

### P1-7: Email Logging Leak (FIXED)

**Files modified**:
- `src/server/routes/auth/password.rs`

**Changes**:
1. Line 17: `info!("Password reset request for email: {}", request.email)` → `info!("Password reset request received")`
2. Line 23: `info!("Password reset token generated for: {}", request.email)` → `info!("Password reset token generated")`
3. Audited all auth route files — no other PII leaks found

### Verification

All fixes verified with:
- `cargo check --all-features` — compilation clean
- `cargo clippy --all-targets --all-features -- -D warnings` — lint clean
- All module tests pass (pg_vector: 39, a2a: 28, rate_limiter: 64, cache: 132, rate_limit middleware: 232)
