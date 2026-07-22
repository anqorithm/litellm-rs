# Task Plan

## Linked Issue

GH-1112 / #1112

## Spec Packet

- Product: `specs/GH1112/product.md`
- Tech: `specs/GH1112/tech.md`
- Approval: current user explicitly approved the design and selected `implx auto` on 2026-07-22.

## 状态

本计划对应 heavy-tier 两阶段流程：先合并本 spec packet，再从合并后的 `origin/main`
建立 implementation PR。任务严格串行处理共享 catalog 文件；任何 implementation slice
只使用一个 writable owner，独立 reviewer 始终只读。

## 实现任务

- [ ] `SP1112-T1` Covers: B-001, B-002, B-003, B-004, B-014, B-015, B-016, B-017. Owner: Google catalog owner. Dependencies: spec PR merged; fresh duplicate evidence and implement route gate allowed. Files: `src/core/providers/mod.rs`, all old `src/core/providers/gemini/models/**`, all new `src/core/providers/google/**`, `src/core/providers/gemini/mod.rs`, `src/utils/ai/models/pricing.rs`, `src/utils/ai/models/utils_tests.rs`. Done when: catalog data/types move to the neutral owner; old Gemini registry/catalog paths and aliases are removed; exact lookup, separate availability overlays, stable ordering, immutable initialization and validation fixtures pass; every pre/post advertised-ID difference has a disposition; pricing values may move unchanged but pricing behavior is not modified. Verify: `cargo test --locked google_model_catalog`、`cargo test --locked model_utils`、`cargo check --locked` 证明所有旧 registry consumer 已迁移、auth/network counter behavior fixtures、`cargo fmt --all -- --check`.

- [ ] `SP1112-T2` Covers: B-002, B-003, B-005, B-008, B-009, B-010, B-017, B-018. Owner: Gemini consumer owner. Dependencies: SP1112-T1 stable head merged or recorded as stack base. Files: `src/core/providers/gemini/provider.rs`, `src/core/providers/gemini/provider_tests.rs`; read-only use of neutral catalog. Done when: Gemini `models()` filters Developer availability in stable order; validation、supported params and mapping consume one shared contract; retired/unverified/missing-contract/unsupported-param fixtures fail before HTTP; no #1108 model refresh, #1111 tool mapping or #1113 pricing behavior is added. Verify: `cargo test --locked gemini_provider`、shared-contract table fixtures、network counter=0 negatives、fmt/check.

- [ ] `SP1112-T3` Covers: B-001, B-002, B-003, B-004, B-005, B-006, B-007, B-008, B-009, B-010, B-014, B-017, B-018. Owner: Vertex consumer owner. Dependencies: SP1112-T2 stable head; no concurrent owner on neutral catalog. Files: `src/core/providers/vertex_ai/mod.rs`, `src/core/providers/vertex_ai/client.rs`, `src/core/providers/vertex_ai/client/url.rs`, `src/core/providers/vertex_ai/client_tests.rs`, `src/core/providers/vertex_ai/common_utils.rs`, `src/core/providers/vertex_ai/tests.rs`, `src/core/providers/vertex_ai/transformers.rs`, `src/core/providers/vertex_ai/transformers/split_tests.rs`; read-only use of neutral catalog. Done when: Google models use exact catalog lookup + Vertex overlay; partner models use exact partner lookup; static Gemini 1.5 `models()` table、substring classification and chat `Custom` fallback are gone; custom base does not bypass gate; shared contract drives supported params、validation and the actual transformer request body; `common_utils::GenerationConfig` remains only a wire DTO and its independent validator is removed or delegates to the shared contract; unknown/fuzzy/contract-invalid requests have auth/network counters zero. Verify: `cargo test --locked vertex_ai_model_exact`、`cargo test --locked vertex_ai_transformer`、`cargo test --locked vertex_ai`、exact-negative and provider-parity behavior fixtures、fmt/check.

- [ ] `SP1112-T4` Covers: B-011, B-012, B-013, B-018. Owner: auth-boundary verification owner. Dependencies: SP1112-T3 stable head. Files: only existing Gemini/Vertex provider test files in the tech manifest; production auth/endpoint files are read-only. Done when: loopback captures prove Developer query-key-only and Vertex Bearer-only paths; rejected model/contract requests do not acquire credentials or send network calls; catalog/error/Debug/Display/log capture never contains sentinel secrets; no catalog dependency on auth/config/reqwest. Verify: `cargo test --locked google_auth_isolation`、adversarial redaction fixtures、`git diff --check`.

- [ ] `SP1112-T5` Covers: B-001, B-002, B-003, B-004, B-005, B-006, B-007, B-008, B-009, B-010, B-011, B-012, B-013, B-014, B-015, B-016, B-017, B-018. Owner: coordinator + independent security reviewer. Dependencies: SP1112-T1 through SP1112-T4 complete on exact implementation head. Files: read-only verification; findings return to owning task. Done when: focused tests、full fmt/check/strict Clippy/test、SpecRail workflow/spec gates、spec-vs-implementation check、CI、review threads and `pr_gate.py` are current and green; reviewer artifact is schema-valid and exact-head; final slice may use `Fixes #1112`, earlier slices only `Refs #1112`. Verify: all tech-spec Test Plan commands plus fresh GitHub evidence and runtime ledger gate.

## 并行拆分

- T1 → T2 → T3 严格串行：它们共享 neutral catalog/consumer contract，禁止并行写。
- T4 只能在 T3 稳定后执行，只写既有 provider tests，不修改 auth/endpoint production code。
- T5 是只读 reviewer/coordinator lane，不写 production/spec 文件。
- 若实现分为 stacked PR：T1/T2/T3 的下游 writable worktree 只有在上游 head、focused
  verification、dirty state 和 overlapping paths 都记录后才创建；上游 head 改变时下游停止。
- 任一 worker 触碰 manifest 外路径、#1108/#1111/#1113 acceptance surface 或共享文件
  owner 冲突时立即停止并提交 spec amendment，不得把扩大范围当作“顺手修复”。

## 验证

- Product invariant set: `B-001..B-018`。
- Task `Covers:` union: `B-001..B-018`；无 orphan 或 undeclared ID。
- Tech manifest: issue=1112、complete=true，包含 neutral owner、旧 catalog 删除路径、
  Gemini/Vertex consumers 与 utility compatibility consumers。
- Spec 阶段：`python3 checks/check_workflow.py --repo . --spec-dir specs/GH1112`、
  `python3 checks/check_workflow.py --repo .`、`git diff --check`。
- Implementation 阶段：focused tests 迭代；exact final head 只执行一次 full fmt/check/strict
  Clippy/test，并保存原始输出到 artifact file。

## Handoff Notes

- Root cause 是三套模型 authority + fuzzy/Custom fallback + 重复 request contract，不是缺少
  更多 model IDs。
- #1112 为 #1108/#1111/#1113 提供 canonical exact-ID 依赖，但不覆盖或关闭它们。
- Developer/Vertex availability 必须独立；无 Vertex 官方证据时 fail closed。
- Gemini query API key 与 Vertex `VertexAuth` Bearer 不得共享 config、header/query helper。
- pricing 值可机械迁移，pricing authority、单位、unknown behavior 留给 #1113。
- tool capability metadata 可迁移，完整 tool round-trip 留给 #1111。
- 不保留旧 registry alias/wrapper；兼容 helper 直接查询 single canonical registry。
