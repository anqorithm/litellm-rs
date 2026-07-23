# Tech Spec

## Linked Issue

GH-1111 / #1111

## Product Spec

见 `specs/GH1111/product.md`。

## Codebase Context

以下锚点已在 `origin/main@671282f265fdf7ba4a5b1c8d0646e175903faabb` 核验。

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| Canonical tool messages | `src/core/types/chat.rs:18-53`、`src/core/types/content.rs:5-65`、`src/core/types/tools.rs:55-74` | 顶层 `tool_calls`/`tool_call_id` 与 parts `ToolUse`/`ToolResult` 都能表达 call/result；function arguments 为 JSON string。 | B-003–B-008 的输入 authority；无需新增 public type。 |
| Gemini Developer request | `src/core/providers/gemini/client.rs:252-337,339-430` | message role 直接映射，tool role 写为 `function`；`ToolUse`/`ToolResult` 明确返回 multimodal error，空 parts 会补空文本。 | 第二回合失败的直接根因。 |
| Gemini Developer unary response | `src/core/providers/gemini/client.rs:457-547` | 已读取 `functionCall`，但空 name/args 被默认值吞掉，call ID 由 candidate/part 生成。 | 保留稳定 ID 思路，改为严格 shared parser。 |
| Gemini capability/dispatch | `src/core/providers/gemini/provider.rs:85-112,126-150,215-313` | model feature 仅在有 tool declaration 时检查；provider 声明 ToolCalling，unary/stream 分别进入 client 和 `GeminiStream`。 | B-005/B-012 的 preflight 与实际 transport。 |
| Router retry/fallback lifecycle | `src/core/router/execute_impl.rs:21-185,333-428`、`src/core/router/tests/execution_tests.rs:168-260`、`src/core/router/tests/fallback_tests.rs:41-128` | router 的 operation closure 会在 unary retry 与跨 model fallback 中再次调用；现有测试覆盖 retry 调度和 fallback 配置，但没有验证 provider tool ledger/partial stream state 的 attempt 隔离。 | B-013 必须由真实 router lifecycle integration fixture 证明，不能只测 helper 重建。 |
| Gemini SSE | `src/core/providers/base/sse/gemini.rs:8-152`、`src/core/providers/gemini/streaming.rs:15-49` | SSE transformer 只提取 text，所有 chunks 的 `tool_calls=None`，未知 finish reason 默认 Stop。 | B-002/B-009/B-014 的 streaming 根因。 |
| Shared provider utilities | `src/core/providers/shared.rs:1-10,69-90` | 有通用 provider helper，但没有 Google tool wire/ledger；把该语义塞入 generic shared 会模糊 owner。 | GH1112 已批准 neutral Google owner，本 issue 不再创建第二个 generic owner。 |
| Vertex actual request path | `src/core/providers/vertex_ai/client.rs:55-116,119-165,269-335` | chat path 使用 `GeminiTransformer` 后由 `make_request` 获取 Bearer；声明 streaming，但 trait 未实现 stream method。 | B-010–B-012 的真实运行路径和认证边界。 |
| Vertex transformer | `src/core/providers/vertex_ai/transformers.rs:19-110,113-190,194-274` | tool declarations 存在；canonical ToolUse/ToolResult 被拒绝，response 只读 text、丢弃 calls。 | 与 Developer 必须共享 semantic contract。 |
| Coverage workflow | `.github/workflows/ci-coverage.yml:3-6,54-55,71-84` | coverage 只在 schedule/manual 跑，installer 使用浮动 `@cargo-llvm-cov`，命令未启用 branch，且尚无 PR changed-line gate；Codecov 设置 `fail_ci_if_error: false`，上传故障可显示为 green。scheduled/full coverage 与 PR/manual immutable-base gate 的职责未分离。 | B-018 必须固定工具版本，分离 trigger 职责，并使所有 required coverage uploads fail-closed。 |
| Vertex wire DTO | `src/core/providers/vertex_ai/common_utils.rs:33-77` | 实际 transformer 使用 `Part::FunctionCall/FunctionResponse`，字段缺少显式 camelCase rename。 | 需要由 adapter 生成正确 `functionCall`/`functionResponse` wire。 |
| Vertex secondary trait path | `src/core/providers/vertex_ai/client.rs:461-627,629-729` | `transform_request` 直接序列化 canonical messages；`transform_response` 只取首个 text part。 | 不修会保留第二条错误语义路径。 |
| Vertex URL/auth | `src/core/providers/vertex_ai/client/url.rs:20-70`、`src/core/providers/vertex_ai/client.rs:79-115` | streaming URL 已支持 `?alt=sse`；request 使用 `VertexAuth` Bearer。 | 可以实现真实 Vertex SSE，同时保持认证隔离。 |
| Vertex tests | `src/core/providers/vertex_ai/transformers/split_tests.rs:1-292`、`src/core/providers/vertex_ai/client_tests.rs:1-140`、`src/core/providers/vertex_ai/tests.rs` | 有 pure transformer、client/capability/auth 基础，但没有完整 tool loop。 | 放置正负 fixture 与 auth/transport 回归。 |
| Upstream neutral owner | `specs/GH1112/tech.md`（PR #1117） | 已批准 `src/core/providers/google/**` 成为 Gemini/Vertex provider-neutral owner，并明确把 tool wire 留给 #1111。 | GH1111 implementation 必须串行基于 GH1112 stable head，不得回到 Gemini-owned/shared duplicate。 |

## Planned Changes

```specrail-planned-changes
{
  "issue": 1111,
  "complete": true,
  "paths": [
    "src/core/providers/google/mod.rs",
    "src/core/providers/google/tool_loop.rs",
    "src/core/providers/gemini/client.rs",
    "src/core/providers/gemini/provider.rs",
    "src/core/providers/gemini/provider_tests.rs",
    "src/core/providers/base/sse/gemini.rs",
    "src/core/providers/gemini/streaming.rs",
    "src/core/providers/vertex_ai/common_utils.rs",
    "src/core/providers/vertex_ai/transformers.rs",
    "src/core/providers/vertex_ai/transformers/split_tests.rs",
    "src/core/providers/vertex_ai/client.rs",
    "src/core/providers/vertex_ai/client/url.rs",
    "src/core/providers/vertex_ai/client_tests.rs",
    "src/core/providers/vertex_ai/streaming.rs",
    "src/core/providers/vertex_ai/mod.rs",
    "src/core/providers/vertex_ai/tests.rs",
    "src/core/router/tests/execution_tests.rs",
    "src/core/router/tests/fallback_tests.rs",
    ".github/workflows/ci-coverage.yml",
    "scripts/guards/check_changed_coverage.py",
    "scripts/guards/coverage/gh1111.json"
  ],
  "spec_refs": [
    "B-001", "B-002", "B-003", "B-004", "B-005", "B-006",
    "B-007", "B-008", "B-009", "B-010", "B-011", "B-012",
    "B-013", "B-014", "B-015", "B-016", "B-017", "B-018"
  ]
}
```

该 manifest 以 GH1112 合并后的 neutral Google owner 为 base gate。若 PR #1117 的最终路径或
API 与上述 `google/mod.rs` 边界不同，先提交 GH1111 spec amendment；不得改为
`shared/gemini_tools.rs`、Gemini-owned helper 或清单外临时模块。`client/url.rs` 仅允许补充
Vertex SSE URL fixture/最小构造调整，认证与 endpoint authority 不迁入 shared helper。

## 设计方案

### 1. 单一 crate-private tool-loop owner

在 GH1112 创建的 `src/core/providers/google/` 下新增 `tool_loop.rs`。模块只接收 canonical
chat/tool types 与 JSON value，不接收 provider config、auth、HTTP client、model catalog 或
endpoint：

- `GoogleToolLedger` 按 message 顺序登记 call ID、tool name、arguments 与消费状态；
- `plan_tool_parts(&[ChatMessage])` 生成按 message index 排列的 neutral
  `GoogleToolPart::{FunctionCall, FunctionResponse}`；
- `parse_function_calls(parts, candidate_index)` 严格解析 provider response；
- `stream_function_call_deltas(parts, candidate_index, stream_state)` 生成稳定 tool-call delta；
- `normalize_tool_result(value, is_error)` 产生可逆 response object。

所有 Rust 字段保持 snake_case；只有 adapter 序列化边界使用显式 `functionCall`、
`functionResponse` 和对应 camelCase 字段。helper 返回 `ProviderError`，provider name 作为闭集
参数传入，禁止 `Any`、warning+fallback 或吞异常。

Ledger 顺序扫描规则：

1. assistant 顶层 `tool_calls` 与 parts `ToolUse` 都可登记，但同一 message 同时用两种表示
   即 ambiguous，整个请求失败；
2. call ID/name 非空且全请求唯一，arguments 必须解析为 object；
3. tool-role 顶层 ID 与 parts `ToolResult` 都可消费，但同一结果不能双表示；
4. result 只解析已经登记且未消费的 ID，name 永远从 ledger 读取；
5. 扫描结束不要求每个 call 都已有 result，允许模型刚产生 call 的合法下一步；已经提供 result
   的 call 必须一对一且无剩余非法表示。

结果 payload 使用唯一规范化函数。`ContentPart::ToolResult.content` 的 object 原样作为
`functionResponse.response`；array/scalar/`null` 进入 `{"result": value}`。若
`is_error=true`，无论原始 shape 都使用 `{"result": original, "is_error": true}`，避免与
用户 object 字段碰撞。带顶层 `tool_call_id` 的 tool-role message 使用以下闭集：

- `MessageContent::Text(text)` → `{"result": text}`，包括空字符串；
- `MessageContent::Parts(parts)` → `{"result": [part_0, ...]}`，每个非工具 part 用现有
  tagged canonical serde shape 序列化并保持输入顺序，空 parts 保留 `[]`；
- parts 内出现 `ToolUse`/`ToolResult`、content 缺失，或顶层/parts 同时表达 result 时，
  作为 B-005 ambiguity/missing-content 在 auth/network 前拒绝。

Gemini 与 Vertex 必须对上述 object、scalar、`null`、Text、ordered Parts、empty Parts、
error 和 ambiguous fixtures 得到 byte-equivalent provider-neutral response object。

### 2. Request adapters

Gemini Developer `transform_chat_request` 和 Vertex `GeminiTransformer` 在处理普通 text/image
前先取得 shared plan：

- assistant call 使用 role=`model` + `functionCall`；
- tool result 使用 role=`user` + `functionResponse`；
- 普通 content 继续由原 adapter 转换，shared helper 不接触 multimodal/auth；
- 同一 message 的 text 与 calls 保持 part 顺序；只有 genuinely empty 的普通 message 才沿用
  既有空文本兼容行为，非法 tool part 不能走该 fallback；
- 全部 validation 在 `make_request`、API key URL 构造和 `VertexAuth::get_access_token` 前完成。

Vertex `common_utils::Part` 对 `FunctionCall` / `FunctionResponse` 使用显式 serde rename，禁止
依赖 Rust variant/field 名碰巧符合 wire。`client.rs::transform_request` 必须委托实际
`GeminiTransformer`，不能继续直接序列化 `ChatMessage`；partner models 保持原路径。

### 3. Unary response parity

Developer client 与 Vertex transformer 都调用 shared strict parser：

- candidate index 优先使用 upstream `index`，缺失时才使用 array position；非法负数/溢出失败；
- part index 与 candidate index 生成 `call_<candidate>_<part>`；
- name 必须非空，args 必须存在且为 object；不再 `unwrap_or("")` / `unwrap_or({})`；
- text 继续按现有顺序拼接，tool calls 单独保持 part order；
- 只要有 tool call，finish reason 固定为 `ToolCalls`；无 tool call 时沿用已声明的 finish mapping，
  未知非空 finish reason 返回 parsing error而不是 Stop。

Vertex `client.rs::transform_response` 委托同一 transformer，避免 secondary trait path 继续丢
calls。未被实际运行路径引用的 `vertex_ai/gemini/mod.rs` 不在本 issue 修改。

### 4. Gemini/Vertex streaming

扩展 `base/sse/gemini.rs::GeminiTransformer`：构造时接收 provider name 与 model，并持有每个
stream 独立的 call state；Developer `GeminiStream` 继续拥有一个 transformer instance。

- 每个 functionCall 首个 delta 写 index、稳定 ID、type、name；后续只追加 arguments；
- 重复完全相同 part 幂等，冲突内容返回 parsing error；
- 有 call 的 terminal chunk 使用 `FinishReason::ToolCalls`；
- malformed chunk、断连、取消不合成成功 terminal。

新增 `vertex_ai/streaming.rs`，复用同一 SSE transformer，但请求仍由 Vertex client：

1. `GeminiTransformer` 先完成 request/ledger validation；
2. `build_url(..., "streamGenerateContent", true)` 保留 `?alt=sse`；
3. `VertexAuth` 获取 Bearer 并发送；
4. response bytes 进入 provider=`vertex_ai` 的 shared SSE transformer；
5. `LLMProvider::chat_completion_stream` 返回该 stream，不再落入默认 NotSupported。

Partner models 不复用 Google SSE path。若 Vertex streaming 在 GH1112 final head 被明确移出
产品能力，本 task 必须先 amendment，而不是静默删掉 B-002/B-012 的验收。

### 5. Capability 与认证边界

- Gemini model validation 对任何含 call/result 的请求执行 ToolCalling feature check，不只检查
  `request.tools.is_some()`；provider-level capability 表示该 provider 有可用模型，model-level
  validation 决定具体 request。
- Vertex `ChatCompletionStream` 只有在上述真实 stream method 存在且 tests 通过时保留；
  ToolCalling 只有 unary 和 streaming 都完成所声明 transport 的回路时保留。
- Developer adapter 不导入 `VertexAuth`；Vertex adapter 不读取 `GeminiConfig.api_key`。
- loopback tests 使用不同 sentinel，断言 query/header 互斥；validation negatives 断言两个
  secret acquisition counter 都为零。

### 6. 错误与 redaction

Request correlation 错误使用 `ProviderError::InvalidRequest`，provider wire 错误使用
`ProviderError::ResponseParsing`。message 只包含 provider、field class、截断/哈希后的 call ID；
禁止序列化完整 args/result、Authorization 或 upstream body。已有 HTTP error mapper 与 retry
policy 保持 authority；shared helper 不决定 retry。

### 7. Router attempt 隔离与 coverage 门禁

Router production ownership 保持在 `execute_impl.rs`，本 issue 默认只读。新增的 integration
fixtures 放入既有 `execution_tests.rs` 与 `fallback_tests.rs`：operation closure 每次被 router
调用时必须从不可变 request snapshot 重新执行 provider preflight 并构造新的 ledger/stream
state。fixture 分别强制一次 pre-output retry 和一次跨 provider fallback，并断言：attempt 使用
不同 ledger identity；旧 attempt 的 partial state 不可见；已消费 result/call 不会重复；只有
尚未产生输出的 attempt 才允许 retry/fallback。若 fixture 暴露真实 router defect，必须先更新
planned-path manifest 并重新审查，不能在清单外顺手修改 `execute_impl.rs`。

新增 `scripts/guards/check_changed_coverage.py`，以 `cargo llvm-cov --branch --lcov` 输出和 immutable
base 相对 exact head 的 changed lines 为输入。`scripts/guards/coverage/gh1111.json` 是 fail-closed
policy，固定 `minimum_changed_line_percent=80`、`critical_branch_percent=100`，并显式列出
correlation registration/consumption、strict provider response parsing、Developer/Vertex
pre-auth validation 和 SSE terminal-state 的 path+symbol branch allowlist。checker 对缺失文件、
缺失 symbol、没有 branch record、低于阈值或无法解析的 LCOV 都非零退出；`--self-test` 使用内置
正负 fixture 验证阈值边界。该 policy 必须随 implementation exact head 更新为实际 symbol，
不得用空 allowlist、忽略缺失 symbol 或人工“查看报告”代替门禁。

同一 T5 必须把 checker 接入 `.github/workflows/ci-coverage.yml`，形成仓库工具链契约：

- installer 固定为
  `taiki-e/install-action@c44f6b046f1c29ae5918b1e0bfdbb2f1813836fd # v2.84.1`，
  并以 `tool: cargo-llvm-cov@0.8.7` 固定工具版本；CI 先断言
  `cargo llvm-cov --version` 精确匹配；
- 保留 schedule/manual，并为 GH1111 manifest、coverage checker/policy 或 workflow 自身变化
  增加有界 `pull_request` path trigger。PR run 的 base 只能使用 immutable
  `${{ github.event.pull_request.base.sha }}`；manual run 必须要求显式 full immutable base SHA input；
- scheduled run 必须执行 full workspace coverage 并上传 LCOV/exact-head metadata，但绝不运行
  changed-line checker，也不要求或推断 base SHA；
- PR 与 manual run 生成 `--branch` LCOV 后必须运行 policy checker。充足 evidence 必须 exit 0；
  missing/malformed/empty/低于 80%/关键 branch 未达 100%，以及缺失或无效 immutable base，均必须非零并使 job 失败；
- 所有 trigger 都上传 LCOV 和 exact-head metadata；PR/manual 还上传 policy result 与 immutable
  base metadata，供独立 reviewer 与 PR gate 绑定同一 SHA。required coverage upload 必须使用
  `fail_ci_if_error: true`；schedule、PR 或 manual 任一上传失败都必须使 required coverage result
  失败，不得依赖 Codecov `target:auto`、warning 或缺失 artifact 显示为 green。

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| B-001 | shared response parser + Developer/Vertex unary adapters | `cargo test --lib --all-features google_tool_unary_order_and_finish_reason` |
| B-002 | shared ID scheme + Gemini/Vertex SSE aggregation | `cargo test --lib --all-features google_tool_unary_stream_equivalence` |
| B-003 | shared request planner + both request adapters | `cargo test --lib --all-features google_tool_call_request_mapping` |
| B-004 | ledger result resolution + role/response adapters | `cargo test --lib --all-features google_tool_result_request_mapping` |
| B-005 | ledger negative matrix before clients | `cargo test --lib --all-features google_tool_correlation_rejects_before_auth_network` |
| B-006 | ordered multi-call ledger | `cargo test --lib --all-features google_parallel_tool_calls_preserve_identity` |
| B-007 | strict args/name validation | `cargo test --lib --all-features google_tool_invalid_arguments_matrix` |
| B-008 | result normalization | `cargo test --lib --all-features google_tool_result_normalization_matrix` |
| B-009 | per-stream state/terminal handling | `cargo test --lib --all-features google_tool_stream_terminal_matrix` |
| B-010 | shared fixture run through both adapters | `cargo test --lib --all-features google_tool_provider_parity` |
| B-011 | separate client/auth loopback capture | `cargo test --lib --all-features google_tool_auth_isolation` |
| B-012 | provider/model capability and actual stream dispatch | `cargo test --lib --all-features google_tool_capability_matches_dispatch` |
| B-013 | per-attempt ledger lifecycle + real router retry/fallback fixtures | `cargo test --lib --all-features google_tool_retry_fallback_fresh_ledger`；fixture 覆盖 pre-output retry 与跨 provider fallback，证明 fresh ledger/stream state、无重复 result/call |
| B-014 | strict malformed provider response matrix | `cargo test --lib --all-features google_tool_malformed_response_matrix` |
| B-015 | existing non-tool and mixed-content fixtures | `cargo test --lib --all-features gemini_provider && cargo test --lib --all-features vertex_ai_transformer` |
| B-016 | crate-private neutral API、删除/停止导出旧 semantic owners、双 provider parity | `cargo check --all-features` 强制 Gemini/Vertex consumers 只依赖 `google/` owner；`cargo test --lib --all-features google_tool_provider_parity`；independent dependency review 核验无第二 ledger/validator。窄 import audit 仅作 advisory，不是完成证据 |
| B-017 | adversarial sentinel/error capture | `cargo test --lib --all-features google_tool_error_redaction` |
| B-018 | trigger-separated, fail-closed coverage evidence: scheduled full coverage/upload; PR/manual changed-line/critical-branch gate + pinned CI toolchain + full deterministic/remote gates | `python3 scripts/guards/check_changed_coverage.py --self-test`；PR/manual 生成 branch LCOV 后以 immutable base SHA 和 `--policy scripts/guards/coverage/gh1111.json` 执行：充足 evidence exit 0，missing/malformed/empty/低阈值/invalid base 非零；scheduled run 不调用 checker 但上传 full LCOV/exact-head artifact；所有 trigger 的 required upload 使用 `fail_ci_if_error: true`，上传失败非零；`ci-coverage.yml` 固定 installer SHA + `cargo-llvm-cov@0.8.7`；final fmt/check/clippy/test；exact-head review、CI、reviewThreads 与 `pr_gate.py` |

## 数据流

Request：`ChatRequest` → model/request preflight → shared ordered tool ledger → provider adapter 普通
parts + neutral tool parts → provider-specific URL/auth → HTTP。任何 ledger/wire error 在 URL secret
注入或 Vertex token acquisition 前返回。

Response：provider JSON/SSE → shared strict functionCall parser/state → canonical `ToolCall` /
`ToolCallDelta` → existing router/callback/usage。下一回合客户端回送 call ID 时，从该请求历史中的
assistant call 重建新 ledger；不持久化全局状态，也不跨 provider/retry 共享。

## 依赖与执行顺序

1. PR #1117 / GH1112 必须先合并并形成 stable neutral Google owner。
2. fresh duplicate evidence 与 implement route gate 必须在 GH1111 implementation branch 前重跑；
   PR #1117 对 #1111 的引用是明确排除/依赖，不是覆盖 PR，但 gate 必须以 live body 复核。
3. shared owner、Developer adapter、SSE、Vertex adapter 严格串行；它们有文件/API 依赖，不能
   伪装成并行 writable lanes。
4. GH1108 model refresh 与 GH1113 pricing 可并行规划，但不得写 GH1111 manifest 中的文件；
   ownership 冲突时停止并 amendment。

## 备选方案

1. **只删掉两个拒绝分支**：拒绝。没有 call ledger、name resolution、strict response parser
   和 streaming，仍会串线或静默丢数据。
2. **Gemini 与 Vertex 各复制一套 helper**：拒绝。会形成新的 semantic drift，违反 #1112
   neutral owner 和 B-016。
3. **放在 generic `shared.rs`/`shared/gemini_tools.rs`**：拒绝。Google-specific wire 会污染
   全 provider utility，且在 #1112 后形成第二 neutral owner。
4. **只修 unary，streaming 保留声明**：拒绝。能力声明仍会路由到空 tool delta或默认
   NotSupported，违反 B-002/B-012。
5. **用 tool name 而非 ID 关联结果**：拒绝。同名并行 calls 会串线。

## 风险

- **Security**：tool result 可能含 secret；error/log/fixture 必须只写安全摘要，auth isolation
  属 SEC-11 人工复审面。
- **Compatibility**：过去被空值 fallback 接受的 malformed response 将改为 error；这是
  明确的 fail-closed 修复，发布说明需列出。
- **Streaming**：重复/分片 functionCall 的 provider wire 可能随 API 变化；固定真实 fixture
  并把冲突当 parsing error，禁止猜测拼接。
- **Dependency**：GH1112 会移动 Vertex/Gemini paths；implementation 前必须重核 manifest。
- **File size**：`gemini/client.rs` 772 行、`vertex_ai/transformers.rs` 762 行接近 800 hard
  ceiling；新增语义必须进入 `google/tool_loop.rs`，修改后两文件不得超过 800 行。若 Vertex
  client 因 stream 超过 ceiling，stream transport 必须在新 `vertex_ai/streaming.rs` 中。
- **Maintenance**：两个 Vertex transform entry points 必须都委托同一 transformer，避免
  一条路径绿、另一条继续丢 call。
- **Evidence drift**：coverage policy 的 symbol/path 必须绑定 exact implementation head；缺失
  symbol 或 branch record 直接失败。router fixtures 是 production lifecycle 证据，helper-only
  test 不能替代。

## 测试计划

- [ ] Shared unit: call/result happy path、并行/同名 calls、object/scalar/error result。
- [ ] Shared negative: empty/unknown/duplicate/mismatch/result-before-call/ambiguous representation。
- [ ] Unary: Gemini 与 Vertex mixed text + multi functionCall golden fixtures。
- [ ] Streaming: Gemini 与 Vertex fragmented/multiple/repeated/malformed/cancel fixtures及 unary parity。
- [ ] Pre-network/auth: invalid request counters 全零。
- [ ] Auth isolation: Developer API key-only；Vertex Bearer-only；sentinel 不出现在 error/log/artifact。
- [ ] Capability: 每个 advertised provider/model/transport 实际 dispatch 成功；partner path 不变。
- [ ] Router lifecycle: `cargo test --lib --all-features google_tool_retry_fallback_fresh_ledger`，覆盖
      pre-output retry 与跨 provider fallback，证明 fresh ledger/stream state 且无重复消费/输出。
- [ ] Regression: `cargo test --lib --all-features gemini_provider`。
- [ ] Regression: `cargo test --lib --all-features vertex_ai_transformer`。
- [ ] Regression: `cargo test --lib --all-features vertex_ai`。
- [ ] Coverage checker self-test: `python3 scripts/guards/check_changed_coverage.py --self-test`。
- [ ] Coverage report: `cargo llvm-cov --all-features --workspace --branch --lcov --output-path artifacts/coverage/GH1111/lcov.info`。
- [ ] PR/manual coverage gate: `python3 scripts/guards/check_changed_coverage.py --lcov artifacts/coverage/GH1111/lcov.info --base "$COVERAGE_BASE_SHA" --policy scripts/guards/coverage/gh1111.json`；充足 exact-head/immutable-base evidence 必须 exit 0；changed lines <80%、policy critical branches <100%、missing/malformed/empty evidence 或 base 无效必须非零。scheduled run 不调用此命令。
- [ ] Coverage CI contract: `.github/workflows/ci-coverage.yml` 使用
      `taiki-e/install-action@c44f6b046f1c29ae5918b1e0bfdbb2f1813836fd` 安装
      `cargo-llvm-cov@0.8.7`；scheduled run 执行 full coverage/upload，PR/manual 执行 branch
      LCOV/checker，并保存相应 head/base/result artifact；三个 trigger 的 required coverage upload
      固定 `fail_ci_if_error: true`，上传失败必须使 job 失败。
- [ ] Deterministic: `cargo fmt --all -- --check`。
- [ ] Build: `cargo check --all-features`。
- [ ] Lint: `cargo clippy --all-targets --all-features -- -D warnings`。
- [ ] Full: `cargo test --all-features`。
- [ ] SpecRail: `python3 checks/check_workflow.py --repo . --spec-dir specs/GH1111`。

## 回滚方案

GH1111 应以一个基于 GH1112 stable head 的 implementation PR 交付。若 shared ledger、任一 adapter
或 streaming/auth regression 失败，整体回滚该 PR，恢复原 fail-closed ToolUse/ToolResult 行为；
不得只回滚 strict validation、保留一半 tool loop 或用 empty/text fallback“恢复兼容”。规格 PR
可独立保留作为后续修复依据，不回滚 GH1112 catalog/auth ownership。
