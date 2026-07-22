# Task Plan

## Linked Issue

GH-1111 / #1111

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`
- Tier: `heavy`
- Draft authorization: current `implx auto` run；不构成 final approval 或 merge waiver。
- Base gate: PR #1117 / GH1112 merged，neutral `src/core/providers/google/**` owner 稳定。
- Route gate: 当前因 open PR #1117 在正文中引用 #1111 而 blocked；只有在 #1117 关闭/合并、
  fresh duplicate evidence 证明无覆盖 PR 且 `route_gate implement` allowed 后才可开始 T1。

## 实现任务

- [ ] `SP1111-T1` Owner: Google tool semantic owner. Covers: B-003, B-004, B-005, B-006, B-007, B-008, B-010, B-013, B-014, B-016, B-017. Dependencies: GH1111 spec PR merged；PR #1117 merged；fresh duplicate evidence；implementation route gate allowed. Files: `src/core/providers/google/mod.rs`, `src/core/providers/google/tool_loop.rs` only. Done when: crate-private ordered ledger、neutral call/response parts、strict unary parser、stable ID generator、result normalization 和 stream state primitives 完成；旧 semantic owner 删除或停止导出，使 Gemini/Vertex 只能编译到这一 owner；所有 invalid correlation/wire 分支返回 typed error，helper 无 auth/config/HTTP/catalog dependency；critical negative branches 100% 覆盖. Verify: `cargo test --lib --all-features google_tool`；`cargo test --lib --all-features google_tool_provider_parity`；`cargo fmt --all -- --check`；`cargo check --all-features`；independent dependency review 核验单一 owner（窄 import scan 仅 advisory）。

- [ ] `SP1111-T2` Owner: Gemini Developer adapter owner. Covers: B-001, B-003, B-004, B-005, B-006, B-007, B-008, B-010, B-011, B-012, B-013, B-014, B-015, B-016, B-017. Dependencies: SP1111-T1 stable head. Files: `src/core/providers/gemini/client.rs`, `src/core/providers/gemini/provider.rs`, `src/core/providers/gemini/provider_tests.rs`. Done when: Developer request/unary response 委托 shared ledger/parser；call/result role 与 wire key 正确；任何含 call/result 的请求执行 model capability preflight；invalid matrix 在 URL secret 注入、network、budget/callback 前失败；`client.rs` ≤800 行且 non-tool regression 不变. Verify: `cargo test --lib --all-features gemini_provider`；`cargo test --lib --all-features google_tool_call_request_mapping`；`cargo test --lib --all-features google_tool_unary_order_and_finish_reason`；`cargo test --lib --all-features google_tool_correlation_rejects_before_auth_network`；fmt/check。

- [ ] `SP1111-T3` Owner: Google SSE owner. Covers: B-001, B-002, B-006, B-007, B-009, B-012, B-013, B-014, B-015, B-017. Dependencies: SP1111-T2 stable head. Files: `src/core/providers/base/sse/gemini.rs`, `src/core/providers/gemini/streaming.rs` only. Done when: per-stream call state 产生稳定 ID/index、单调 arguments delta 与唯一 ToolCalls terminal；text + calls 保留；duplicate/conflicting/malformed chunks、取消和断连不伪完成；Developer unary/stream 聚合等价. Verify: `cargo test --lib --all-features google_tool_unary_stream_equivalence`；`cargo test --lib --all-features google_tool_stream_terminal_matrix`；`cargo test --lib --all-features gemini_streaming`；fmt/check。

- [ ] `SP1111-T4` Owner: Vertex adapter/transport owner. Covers: B-001, B-002, B-003, B-004, B-005, B-006, B-007, B-008, B-009, B-010, B-011, B-012, B-013, B-014, B-015, B-016, B-017. Dependencies: SP1111-T3 stable head；GH1112 Vertex consumer paths已落地. Files: `src/core/providers/vertex_ai/common_utils.rs`, `src/core/providers/vertex_ai/transformers.rs`, `src/core/providers/vertex_ai/transformers/split_tests.rs`, `src/core/providers/vertex_ai/client.rs`, `src/core/providers/vertex_ai/client/url.rs`, `src/core/providers/vertex_ai/client_tests.rs`, `src/core/providers/vertex_ai/streaming.rs`, `src/core/providers/vertex_ai/mod.rs`, `src/core/providers/vertex_ai/tests.rs`. Done when: actual unary 与 secondary trait path 都委托 shared transformer；wire 使用 `functionCall`/`functionResponse`；真实 `chat_completion_stream` 使用 Vertex URL + Bearer 并复用 shared SSE semantics；partner path 不变；Developer key/Vertex Bearer 双 sentinel 互斥且 invalid preflight token/network counter=0；`transformers.rs` ≤800 行. Verify: `cargo test --lib --all-features vertex_ai_transformer`；`cargo test --lib --all-features google_tool_provider_parity`；`cargo test --lib --all-features google_tool_auth_isolation`；`cargo test --lib --all-features google_tool_capability_matches_dispatch`；`cargo test --lib --all-features vertex_ai`；fmt/check。

- [ ] `SP1111-T5` Owner: verification/security owner. Covers: B-005, B-007, B-008, B-009, B-011, B-013, B-014, B-017, B-018. Dependencies: SP1111-T1 through T4 complete on one exact head. Files: `src/core/router/tests/execution_tests.rs`, `src/core/router/tests/fallback_tests.rs`, `scripts/guards/check_changed_coverage.py`, `scripts/guards/coverage/gh1111.json` plus existing provider test files from the tech manifest；`src/core/router/execute_impl.rs` remains read-only unless a failing fixture first proves a defect and the spec manifest is amended/re-reviewed；production findings return to the owning task. Done when: exhaustive request/response/stream negative matrix、pre-network/auth counters、secret/tool-output redaction、retry/cancel complete；真实 pre-output retry 与跨 provider fallback 每次重建 ledger/stream state 且不重复消费/输出；coverage checker 对缺失/低阈值 evidence fail closed；changed lines ≥80%，policy 明列的 correlation/invalid-wire/auth-isolation/terminal branches 100%. Verify: `cargo test --lib --all-features google_tool`；`cargo test --lib --all-features google_tool_retry_fallback_fresh_ledger`；`python3 scripts/guards/check_changed_coverage.py --self-test`；`cargo llvm-cov --all-features --workspace --branch --lcov --output-path artifacts/coverage/GH1111/lcov.info`；`python3 scripts/guards/check_changed_coverage.py --lcov artifacts/coverage/GH1111/lcov.info --base origin/main --policy scripts/guards/coverage/gh1111.json`。

## 并行拆分

- T1 → T2 → T3 → T4 严格串行：T2 消费 T1 API，T3 消费同一 parser/state，T4 同时依赖
  neutral API 与 SSE API。即使文件表面不重叠，也不得并行 writable worktree。
- T5 只在 exact implementation head 上执行；独立 reviewer 可与 T5 的只读 coverage 分析并行，
  但两者都不得写 production files。
- 每个 worker 只写本 task 的 Files；`specs/GH1111/**`、`AGENTS.md`、workflow/gate files 和
  GH1112 catalog/request-contract paths默认 forbidden。发现清单外 production path 先 amendment。
- 共享全量 cargo verification 由 coordinator 串行执行；禁止同一 worktree 并发 cargo。

## 验证

- [ ] `SP1111-T6` Owner: coordinator + independent reviewer/merge-reviewer. Covers: B-001, B-002, B-003, B-004, B-005, B-006, B-007, B-008, B-009, B-010, B-011, B-012, B-013, B-014, B-015, B-016, B-017, B-018. Dependencies: SP1111-T1 through T5 complete. Files: read-only verification；findings return to owning task. Done when: product B-001..B-018 与 diff/tests 一一核对；planned-path manifest 无遗漏/越界；compile/type ownership + provider parity + independent dependency review 证明 B-016；coverage guard 以 exact-head LCOV 非零门禁证明 B-018；fresh focused tests、fmt/check/strict Clippy/full test、SpecRail workflow/spec checks、independent exact-head review、GitHub CI、reviewThreads、merge state、`pr_gate.py` 与 runtime ledger 均 current/green；final implementation slice 使用 `Fixes #1111`，spec PR 只用 `Refs #1111`. Verify: `python3 scripts/guards/check_changed_coverage.py --self-test`；`python3 scripts/guards/check_changed_coverage.py --lcov artifacts/coverage/GH1111/lcov.info --base origin/main --policy scripts/guards/coverage/gh1111.json`；`cargo fmt --all -- --check && cargo check --all-features && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-features`；`python3 checks/check_workflow.py --repo .`；`python3 checks/check_workflow.py --repo . --spec-dir specs/GH1111`；fresh spec-vs-implementation、review JSON、GitHub evidence 与 PR gate commands。

## Handoff Notes

- Root cause 不是“少两个 match arm”，而是 request correlation、strict response parsing、SSE
  delta、Vertex secondary path 与 capability 声明没有同一 semantic owner。
- PR #1117 当前对 #1111 的 reference 是明确排除/依赖，但 duplicate adapter 仍会 fail closed；
  不得伪造 evidence 绕过。#1117 合并/关闭后重跑 adapter 与 prior rejection gate。
- #1108 只处理 model refresh/request parameters，#1113 只处理 pricing；不得顺手带入。
- Developer API key 与 Vertex Bearer 的交叉依赖是 SEC-11 阻断项，必须独立人工安全审查。
- 任何 implementation 发现 GH1112 final neutral owner 不兼容本 manifest，先更新 tech/tasks 并
  重新审查；不得退回 generic shared 或复制两套 helper。
- 本 packet 在 auto mode 可被起草并开 spec PR，但 heavy implementation 仍遵守全部
  duplicate/route/reviewer/CI/review-thread/pr_gate/runtime-ledger 门禁。
