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

- [ ] `SP1113-T1` Covers: B-001, B-003, B-010. Owner: dependency/inventory owner. Dependencies: this spec PR merged；PR #1117 merged；fresh `origin/main` and duplicate evidence. Files: read-only `src/core/providers/google/models/**`, then spec manifest only if merged paths changed. Done when: exact GH1112 API/SHA recorded；all Google public surfaces include providerless `PricingService`、Provider facade、Gemini/ModelUtils、`core::cost`、PricingDatabase、`/v1/pricing`；catalog price metadata remains read-only with a narrowly documented guard exemption and cannot reach live fallback/user-visible authority；all seven reviewed owners appear in manifest/tasks. Verify: `rg` inventory/call graph；catalog byte-for-byte no-diff assertion；guard catches duplicate calculators outside catalog metadata；SpecRail checks；independent manifest review.

- [ ] `SP1113-T2` Covers: B-001, B-002, B-004, B-005, B-007, B-009, B-010. Owner: exact authority + typed discriminator/service carrier owner. Dependencies: SP1113-T1. Files: existing T2 manifest paths plus `src/core/pricing_service/service.rs`, `src/core/pricing_service/service_tests.rs`, `src/core/pricing_service/types.rs`；GH1112 catalog read-only. Done when: exact surface-aware resolver remains sole authority；providerless Google APIs require determinable surface or typed fail；`PricingUsage` carries video losslessly；embedded adapter fails typed；non-Google behavior unchanged. Verify: `cargo test --locked google_pricing_exact`；`cargo test --locked google_pricing_units` video matrix；`cargo test --locked pricing_service` providerless surface matrix；`cargo test --locked cost::calculator`；guards；fmt/check.

- [ ] `SP1113-T3` Covers: B-001, B-002, B-003, B-004, B-005, B-009, B-010. Owner: public helper/API owner. Dependencies: SP1113-T2. Files: existing T3 manifest paths plus `src/core/providers/mod.rs`. Done when: helpers thin-delegate typed authority；multimodal video uses T2 carrier；Google facade preserves prefix for GH1112 exact validation while non-Google stripping remains unchanged；ambiguous surface fails；no fuzzy/zero/local math. Verify: existing T3 commands plus Google prefix/unknown-prefix/video matrix and non-Google facade compatibility；fmt/check.

- [ ] `SP1113-T4` Covers: B-005, B-006, B-007, B-008, B-009. Owner: runtime policy/audit + terminal identity owner. Dependencies: SP1113-T3. Files: existing T4 manifest paths plus `src/core/providers/unified_provider_http_mapping.rs`, `src/server/routes/ai/chat.rs`, `src/server/routes/ai/completions_streaming.rs`, `src/server/routes/ai/gemini/provider.rs`, `src/server/routes/ai/gemini.rs`. Done when: chat unary/streaming threads settlement priced/fallback cost into callback；Gemini unary/streaming uses actual selected pricing identity without selection/fallback changes；callback never recalculates；HTTP mapping recognizes only reserved pricing provider, never message prefix；typed Reject/AllowUnpriced audit behavior remains intact. Verify: `cargo test --locked google_pricing_runtime_parity` unary/streaming/Gemini identity matrix；`cargo test --locked allow_unpriced_audit_matrix`；callback/image/retry commands；HTTP reserved-provider positive + ordinary-prefix negative；guards；fmt/check.

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
- Rollback follows owners: T2 authority/service/video carrier together；T3 Google facade/helpers
  together while preserving non-Google prefix semantics；T4 HTTP mapping and callback/selected
  identity threading together。Catalog metadata is never part of implementation or rollback diff.

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
