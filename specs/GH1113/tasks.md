# Task Plan

## Linked Issue

GH-1113 / #1113

## Spec Packet

- Product: `specs/GH1113/product.md`
- Tech: `specs/GH1113/tech.md`
- Architecture decision: typed `Result` for public cost helpers；`AllowUnpriced` only as an
  explicit audited gateway policy.

## 状态

本 packet 只批准设计与任务拆分。Implementation 必须等待 PR #1117 合并，从其 exact merged
head 重锚 Google catalog paths/symbols，并重新通过 implement route gate。未满足依赖时不得
在当前分支写 production code。

## 实现任务

- [ ] `SP1113-T1` Covers: B-001, B-003, B-010. Owner: dependency/inventory owner. Dependencies: this spec PR merged；PR #1117 merged；fresh `origin/main` and duplicate evidence. Files: read-only scan of merged `src/core/providers/google/models/**`, then spec manifest only if its planned paths changed. Done when: exact GH1112 canonical-ID/availability API and merge SHA recorded；all Google public token-cost/price-lookup surfaces have a disposition, including Gemini module/ModelUtils、`core::cost`、shared `PricingDatabase` and `/v1/pricing`；provider image/TTS/partner helpers explicitly excluded while normal image route policy ownership remains included；source guard design finds hard-coded Gemini price tuples、substring/default branches、`unwrap_or(0.0)`、HTTP success-zero and any new helper；manifest matches merged paths. Verify: `rg` inventory + call graph；`git log --merges`/GitHub merge evidence；`python3 checks/check_workflow.py --repo . --spec-dir specs/GH1113`；independent manifest review.

- [ ] `SP1113-T2` Covers: B-001, B-002, B-004, B-005, B-007, B-009, B-010. Owner: exact pricing authority + internal typed-discriminator owner. Dependencies: SP1113-T1 stable head. Files: `src/core/pricing_service/authority.rs`, `src/core/pricing_service/authority_tests.rs`, `src/core/pricing_service/mod.rs`, `src/core/cost/calculator.rs`, `src/core/cost/calculator/pricing.rs`, `src/core/cost/calculator/tests/edge_case_tests.rs`, `src/core/cost/calculator/tests/pricing_lookup_tests.rs`；merged GH1112 Google registry is read-only. Done when: `PricingService` has one Developer/Vertex surface-aware exact canonical resolver over the loaded runtime source；Google path cannot enter fuzzy/longest/default/cross-surface fallback；token-priced records always validate both input/output price fields even for input-only/output-only/zero usage；missing/one-sided/invalid pricing returns a closed crate-private `PricingFailureKind` available to request-time policy；public helper mapping uses only existing `ModelNotFound`/`Configuration` variants and the exhaustive public `ProviderError` enum is unchanged；USD/token conversion has one owner；embedded compatibility initialization returns a typed error instead of constructing an empty service；Gemini/Vertex cannot fall through to `calculator/pricing.rs` hard-coded family tuples；non-Google resolver behavior unchanged. Verify: `cargo test --locked google_pricing_exact`；`cargo test --locked google_pricing_units`；`cargo test --locked pricing_service`；`cargo test --locked cost::calculator`；internal-kind/no-public-variant/source/branch guard；fmt/check.

- [ ] `SP1113-T3` Covers: B-002, B-003, B-004, B-005, B-009, B-010. Owner: public helper/API migration owner. Dependencies: SP1113-T2 stable head. Files: `src/core/providers/gemini/mod.rs`, `src/core/providers/gemini/models/mod.rs`, `src/core/providers/gemini/provider.rs`, `src/core/providers/gemini/provider_tests.rs`, `src/core/providers/vertex_ai/client.rs`, `src/core/providers/vertex_ai/client_tests.rs`, `src/core/providers/vertex_ai/gemini/mod.rs`, `src/core/providers/vertex_ai/tests.rs`, `src/utils/ai/models/pricing.rs`, `src/core/pricing.rs`, `src/core/pricing/tests.rs`, `src/server/routes/pricing.rs`, `CHANGELOG.md`. Done when: Gemini inherent/basic/multimodal/module/ModelUtils、Vertex Gemini、Vertex provider、shared pricing DB Google branches、`/v1/pricing` and already-typed trait/facade all thin-delegate to canonical authority and expose typed failure；Google input without a determinable Developer/Vertex surface fails instead of guessing；unknown/default/zero/substrings、HTTP success-zero and helper-local unit math are removed or unreachable；migration note has old/new signatures；zero usage validates pricing before `Ok(0.0)`；non-Google database/API compatibility and partner/image/TTS provider behavior remain unchanged. Verify: compile-time public signature fixtures；`cargo test --locked gemini_provider`；`cargo test --locked vertex_ai`；`cargo test --locked google_public_cost_helpers`；`cargo test --locked pricing_route`；`cargo test --locked core::pricing`；unit/source guards；fmt/check.

- [ ] `SP1113-T4` Covers: B-005, B-006, B-007, B-008, B-009. Owner: runtime policy/audit owner. Dependencies: SP1113-T3 stable head. Files: `src/server/routes/ai/callbacks.rs`, `src/server/routes/ai/spend.rs`, `src/server/routes/ai/spend/pricing.rs`, `src/server/routes/ai/spend/completion.rs`, `src/server/routes/ai/spend/unpriced.rs`, `src/server/routes/ai/gemini/spend.rs`, `src/server/routes/ai/audio/budgeting.rs`, `src/server/routes/ai/images/generation.rs`, `src/server/routes/ai/images/proxy_spend.rs`, `src/server/routes/ai/response_cache.rs`, `src/server/routes/ai/spend_runtime_pricing_tests.rs`, `src/server/routes/ai/spend_tests.rs`, `src/server/routes/ai/execution_retry_delay_tests.rs`. Done when: selected provider/canonical model/source matches reservation、normal/proxy image settlement、spend and callback；callback consumes the same priced breakdown or explicit-unpriced fallback cost instead of independently recalculating and dropping cost；default Reject stops before upstream/mutation/success effects；policy functions accept only crate-private `PricingFailureKind` before public-error mapping and contain no Display/message-prefix parsing；completion/usage/Gemini/audio/both image callsites mechanically preserve existing policy behavior；public helper errors use existing variants only；an ordinary `InvalidRequest` carrying the legacy prefix cannot enter fallback；only internal typed failure + explicit AllowUnpriced enters fallback；positive and explicit-zero fallback preserve provider/model/policy/outcome/cost in metrics、structured error log、callback and key-scoped `UsageRecord::unpriced`；other errors never bypass；Reject retry recognition uses existing variant + reserved provider field, not message；record failure emits error；retry/fallback re-resolves each candidate. Verify: `cargo test --locked google_pricing_runtime_parity`；`cargo test --locked allow_unpriced_audit_matrix`；`cargo test --locked callbacks`；`cargo test --locked image_generation`；`cargo test --locked execution_retry_delay`；message-prefix/no-public-variant negative/source guard；counter/redaction fixtures；fmt/check.

- [ ] `SP1113-T5` Covers: B-001, B-002, B-003, B-004, B-005, B-006, B-007, B-008, B-009, B-010. Owner: verification coordinator + independent reviewer. Dependencies: SP1113-T1 through T4 complete on one exact head. Files: read-only verification；findings return to owning task. Done when: product/spec/task mapping has no gap；manifest exact and no scope drift；focused tests、source guards、branch coverage、fmt/check/strict Clippy/full test pass；new lines ≥80% and exact/unit/error/policy/audit critical branches 100%；spec-vs-implementation、independent exact-head review、GitHub CI、review threads、merge state、PR gate and runtime ledger are fresh/green；final implementation PR uses `Fixes #1113`. Verify: all tech Test Plan commands plus exact-head LCOV/policy artifact and fresh GitHub evidence.

## 并行拆分

- T1 → T2 → T3 → T4 严格串行：它们共享 catalog/authority/consumer contract and cannot
  infer a downstream shape before the upstream stable head.
- T2 owns pricing authority、core-cost compatibility adapter/fallback and structured error files；
  T3 exclusively owns provider/public helper、shared pricing DB and `/v1/pricing` files；T4 owns
  gateway policy、normal/proxy image、callback classifier/tests and `spend/unpriced.rs`；all consume
  the merged Google registry read-only.
- T5 is read-only. Any reviewer finding returns to its owning task；reviewer cannot edit, approve,
  resolve hosted threads or merge.
- Any change outside the tech manifest, any GH1112 path mismatch, or any request to alter auth、
  model refresh、provider selection、persistence requires a reviewed spec amendment.

## 验证

- Product invariant set: `B-001..B-010`.
- Task `Covers:` union: `B-001..B-010`; no orphan or undeclared ID.
- Spec stage:
  `python3 checks/check_workflow.py --repo . --spec-dir specs/GH1113`;
  `python3 checks/check_workflow.py --repo .`;
  `git diff --check`.
- Implementation focused:
  `cargo test --locked google_pricing_exact`;
  `cargo test --locked google_pricing_units`;
  `cargo test --locked google_public_cost_helpers`;
  `cargo test --locked google_pricing_runtime_parity`;
  `cargo test --locked allow_unpriced_audit_matrix`.
- Final:
  `cargo fmt --all -- --check`;
  `cargo check --all-targets --all-features --locked`;
  `cargo clippy --all-targets --all-features --locked -- -D warnings`;
  `cargo test --all-features --locked -- --test-threads=1`;
  exact-head fail-closed branch coverage gate；SpecRail review/PR/runtime gates.

## Handoff Notes

- Root cause is duplicate price authority plus loss of typed failure, not missing model IDs.
- GH1112 owns canonical ID、availability、request contract and auth isolation；GH1113 consumes
  those facts and owns pricing semantics/units/errors only.
- GH1108 owns price/model refresh；do not add guessed values here.
- Public compatibility direction is already chosen: typed `Result`, no `Option`/bare-f64 legacy
  wrapper that can revive zero/default success.
- `AllowUnpriced` remains explicit request-time policy with unpriced audit evidence；provider
  helpers never interpret it.
