# Product Spec

## Linked Issue

GH-1111 / #1111

complexity: large

## 用户问题

`litellm-rs` 对 Gemini Developer API 和 Vertex AI 声明了工具调用能力，但当前请求转换会
拒绝 `ContentPart::ToolUse` / `ContentPart::ToolResult`，Vertex 还会把 canonical
`ChatMessage` 直接当作 Gemini wire body。调用方能够收到第一次 `functionCall`，却不能把
工具结果可靠地送回模型，导致 agent/tool loop 在第二回合前失败。

用户需要一条完整、可验证且 fail-closed 的工具调用回路：模型产生的 call 在 unary 与
streaming 中映射为稳定的 canonical `ToolCall`；调用方回送的结果按 call ID 关联原始工具名，
再映射为 Gemini `functionResponse`。Gemini Developer API 与 Vertex AI 必须共享相同的
工具语义，同时保持 API key/query 与 Bearer/Vertex endpoint 两套认证边界完全隔离。

## 目标

- 补齐 assistant tool call → 调用方 tool result → 下一轮模型响应的完整闭环。
- 同时支持 canonical 顶层 `tool_calls` / `tool_call_id` 与 `ContentPart::ToolUse` /
  `ContentPart::ToolResult` 两种已有表示，且不允许歧义或重复表示静默覆盖。
- Gemini Developer API 与 Vertex AI 复用一份 provider-neutral Google tool-loop 契约。
- Gemini Developer API 与 Vertex AI 的 unary、streaming tool-call 输出具有可聚合等价语义。
- 任何 call correlation、wire payload 或能力不一致都在上游 HTTP 前显式失败。
- 保持现有 text、image、reasoning、tool declaration、预算、重试和错误红线不变。

## 非目标

- 不刷新模型目录、alias、request parameter allowlist 或 lifecycle；这些属于 #1108。
- 不改变 canonical Google model catalog、exact lookup 或 availability overlay；这些属于
  #1112，GH1111 implementation 必须在其稳定 head 上串行落地。
- 不收敛 pricing authority 或未知模型成本语义；这些属于 #1113。
- 不在 gateway 内执行工具，也不新增 tool registry、router 或 agent runtime。
- 不扩展 partner model、batch product、Files、embeddings 或 image generation 行为。
- 不合并 Vertex 中当前未被运行路径引用的重复 wire DTO；只修改实际 chat 调用链。
- 本规格 PR 不实现生产代码、不声明最终批准，也不绕过后续 CI/review/pr_gate。

## Behavior Invariants

1. **B-001** 当 Gemini/Vertex 响应包含一个或多个合法 `functionCall` part 时，每个 part
   必须映射为一个 canonical `ToolCall`，保留 candidate 顺序、part 顺序、非空名称和完整
   JSON arguments；非空 upstream `functionCall.id` 必须逐字保留，缺失时才可合成 ID；每个 call
   的 opaque thought signature（若存在）必须附着于对应 canonical call 并可在后续请求中原样
   replay，不得检查或改写；存在 tool call 的 choice 必须使用 `finish_reason=tool_calls`。
2. **B-002** 同一 provider 响应经 unary 解析与 streaming 聚合后，必须得到相同的 choice
   index、tool-call index、call ID、名称、arguments、文本和最终 `finish_reason`；call ID
   优先使用非空 upstream ID；仅在缺失时，才由稳定的 message/turn、candidate 与 part 身份
   合成，且在整个请求历史中唯一，不得依赖时间、随机数或 chunk 到达顺序。
3. **B-003** canonical assistant `tool_calls` 或 `ContentPart::ToolUse` 必须按原始顺序映射为
   Gemini model-role `functionCall`，保留非空 call ID、名称、JSON input 与可选 opaque
   thought signature；一次 call 恰好产生一个 wire part，result replay 必须使用同一 upstream
   或 fallback ID。canonical `Tool` 必须在 Gemini Developer wire 中序列化为
   `tools[].functionDeclarations`；Vertex 的 Tool/ToolConfig wire key 必须为 camelCase。
   不支持或 malformed 的 declaration shape 必须在 auth/network 前失败。
   `ChatMessage.function_call` 与 `ChatRequest.functions`/`function_call` 必须规范化到同一
   declaration/call ledger，或在 auth/network 前显式拒绝，不得静默忽略。
4. **B-004** canonical tool-role `tool_call_id` 或 `ContentPart::ToolResult.tool_use_id` 必须
   唯一关联同一请求历史中先前出现的 call，并映射为 user-role `functionResponse`；wire
   name 必须来自被关联的 call，不能从结果消息猜测或使用空字符串。
5. **B-005** 缺失或空 call ID、未知 ID、重复 call ID、重复消费同一结果、结果早于 call、
   call/result 类型不匹配、legacy 与 modern 表示歧义、同一语义同时出现在顶层与 content
   parts 等客户端非法状态，必须返回 typed 4xx/`ProviderError::InvalidRequest`；上游请求、
   credential acquisition、成功 callback、retry/fallback 计数均为零，且任何 deployment
   failure count、cooldown 与 health state 必须保持不变。
6. **B-006** 多个并行 calls 与 results 必须保持一对一关联和确定顺序；相同工具名的不同
   call 不得串线，结果到达顺序不得覆盖 call identity，`parallel_tool_calls=false` 也不得
   被实现层擅自改成并行：必须映射为禁止并行的 provider 配置，或在 auth/network 前返回
   typed `InvalidRequest`；absent/true/false 均必须有明确 disposition。
7. **B-007** `functionCall.args`、`ToolUse.input` 和 function-style arguments 必须是合法的
   JSON object；缺失、`null`、scalar、array、无法解析的 JSON 或空 tool name 必须显式
   失败，不得补成 `{}`、空名称或普通文本成功。
8. **B-008** `ContentPart::ToolResult.content` 为 object 时保持字段和值；array、scalar 或
   `null` 必须通过稳定的 `result` envelope 无损表达。带顶层 `tool_call_id` 的 tool-role
   message 若为 `MessageContent::Text(text)`，必须规范化为 `{"result": text}`；若为
   `MessageContent::Parts(parts)`，必须规范化为 `{"result": [part_0, ...]}`，每个允许的
   canonical part 保留显式 type/字段和值且顺序不变，空 parts 保留为空数组。顶层 ID 与
   parts 内嵌 `ToolUse`/`ToolResult` 属于重复/歧义表示并按 B-005 拒绝；缺失 content 也
   显式失败。`is_error=true` 必须以 `{"result": original, "is_error": true}` 保留原内容
   并产生稳定错误标记；`is_error=false` 与缺失按既有 canonical 语义处理，不得删除结果。
9. **B-009** streaming 可以分多个 chunks 传递文本与多个 calls，但同一 call 只能有一次
   身份创建、单调 arguments delta 和一个终态；重复上游 part、取消、断连或 parser error
   不得产生重复 call、伪 `tool_calls` 完成或随后成功终态。
10. **B-010** Gemini Developer API 与 Vertex AI 对相同 canonical 请求/响应 fixture 必须
    得出相同的 tool-loop 成功或拒绝结论；差异只允许存在于 endpoint、transport 和认证，
    不允许存在于 call correlation、payload normalization 或错误边界。
11. **B-011** Gemini Developer API 请求只能使用其既有 API-key query/header 规则，不得
    获取或发送 Vertex Bearer；Vertex 请求只能使用 `VertexAuth` Bearer，不得读取、记录或
    发送 Gemini API key。任何 tool validation 失败都必须发生在上述 secret 获取之前。
12. **B-012** provider/model 只有在当前 transport 能完成 declaration → call response →
    next-turn result 的闭环时才可声明 `ToolCalling`；声明 streaming 的路径必须真正实现
    `chat_completion_stream`，不得在路由后落入默认 `NotSupported` 或把 SSE 当 unary JSON。
    Gemini Developer 必须发送 `tools[].functionDeclarations`；Vertex 必须发送 camelCase
    `functionDeclarations`、`toolConfig.functionCallingConfig` 与 `allowedFunctionNames`。
13. **B-013** 同一请求的 tool call/result preflight 必须在每次发送及 retry 前重放；retry
    或 fallback 不得复用部分 stream、重复发送已消费 result、重复 tool call 或跨 provider
    复用旧 ledger。
14. **B-014** malformed provider response（空 name、缺失/非法 args、错误 part shape、非法
    candidate index）必须返回 typed response-parsing error；不得使用空值、`STOP` 或 text-only
    fallback 把错误响应伪装成成功。
15. **B-015** 不含工具的 text/image/reasoning 请求、tool declaration、usage 与现有 finish
    reason 行为保持兼容；合法的混合 text + functionCall 响应同时保留文本和 calls，不因其中
    一类存在而丢弃另一类。`tool_choice` 的 absent、`auto`、`none`、合法 forced function
    必须映射为 provider ToolConfig 语义；未知、malformed、forced name 未声明等值必须在
    auth/network 前拒绝。任何 legacy function 字段也必须按 B-003 map-or-reject。
16. **B-016** tool-loop helper 必须是 crate-private 的单一 Google semantic owner；Gemini
    与 Vertex adapter 只能负责各自 wire/transport/auth，不得新增第二套 provider dispatch、
    model catalog、call ledger 或复制一份稍有差异的 validation。
17. **B-017** 对外错误、Debug/Display、日志和测试 artifact 不得包含 API key、Bearer、
    Authorization header 或完整敏感 tool output；诊断只保留 provider、字段类别、call ID 的
    安全摘要和稳定 error kind。
18. **B-018** 完成证据必须包含正负 fixture、pre-network/auth counters、unary/stream
    聚合等价、Gemini/Vertex auth isolation、现有非工具回归以及新增代码 line coverage ≥80%；
    call correlation、invalid wire、secret isolation 和 terminal-state 分支为 100%。scheduled run
    必须执行 full coverage 并上传报告，且不得执行 changed-line checker；`pull_request` 与
    `workflow_dispatch`/manual run 必须用 immutable base 执行 changed-line gate，缺失或无效 base
    必须失败，不能降级为 scheduled/full-coverage 成功。三个 trigger 的 required coverage upload
    都必须 fail-closed：上传失败不得产生 green required coverage result。

## 验收标准

- [ ] Gemini Developer API 与 Vertex AI 都通过单 call、多 call、多 result 的两回合 golden
      request/response fixture，wire 中使用正确的 `functionCall` / `functionResponse` 和 role。
- [ ] 缺失/空/未知/重复/mismatched ID、result-before-call、重复表示、非法 name/args/result
      matrix 全部在上游与 credential counter 为零时失败，且 deployment failure/cooldown/health
      state 不变。
- [ ] Gemini wire golden fixture 证明 canonical declarations 使用
      `tools[].functionDeclarations`；Vertex wire 仅使用 camelCase `functionDeclarations`、
      `toolConfig.functionCallingConfig`、`allowedFunctionNames`，不出现对应 snake_case key。
- [ ] upstream functionCall ID 与 thought signature 在 unary、stream、canonical serde 和下一轮
      replay 中逐 call 原样保留；缺失 ID 的 fallback 在跨两个 assistant turns 时稳定且不同。
- [ ] `parallel_tool_calls` absent/true/false、`tool_choice` 全闭集以及 legacy
      `functions`/`function_call` 的 map-or-reject matrix 均在双 provider 上通过。
- [ ] Gemini 与 Vertex streaming fixture 均能聚合为对应 unary fixture 的同一 ordered
      `ToolCall` 列表；取消、断连与 malformed chunk 不产生伪完成。
- [ ] capability fixture 证明每个声明的 provider/model/transport 都能执行完整 tool round
      trip；未实现的 transport 不再被声明为支持。
- [ ] loopback capture 证明 Developer 只有 API key 路径、Vertex 只有 Bearer 路径，错误和
      artifact 均不含 sentinel secret 或完整 tool output。
- [ ] 现有 text/image/reasoning、usage、budget、retry、tool declaration 与 provider routing
      回归测试保持通过。
- [ ] scheduled coverage run 完整生成并上传 coverage evidence，且不运行 changed-line checker；
      PR 与 manual coverage run 在各自 immutable base 上执行 changed-line/critical-branch gate，
      base 缺失、无效或 coverage evidence 不足均失败；schedule、PR 与 manual 的 required
      coverage upload 失败也均失败，不得 green。
- [ ] `cargo fmt --all -- --check`、`cargo check --all-features`、strict Clippy、全量测试、
      SpecRail workflow/spec gate、独立 reviewer、CI、review threads 与 `pr_gate` 全绿。

## 边界检查

| 边界类别 | 判定 |
| --- | --- |
| Empty / missing input | covered: B-005、B-007、B-014。空 ID/name/args 与缺失 part 都 fail closed。 |
| Error and failure paths | covered: B-005、B-007、B-009、B-014、B-017。请求、响应、stream 与 redaction 均有显式错误。 |
| Authorization / permission | covered: B-005、B-011、B-017。失败发生在 secret acquisition/network 前，认证不可串线。 |
| Concurrency / race / ordering | covered: B-001、B-002、B-006、B-009。candidate/part/chunk/parallel order 和 identity 被固定。 |
| Retry / repetition / idempotency | covered: B-005、B-009、B-013。重复 ID/result/chunk/retry 不得重复执行或完成。 |
| Illegal state transitions | covered: B-004、B-005、B-009、B-012。result-before-call、重复消费和未实现 capability 均被阻断。 |
| Compatibility / migration | covered: B-010、B-015、B-016。双 provider 语义一致，非工具行为与 canonical runtime 保持。 |
| Degradation / fallback | covered: B-005、B-007、B-012、B-014。无空值、text-only、默认 NotSupported 或跨 provider fallback。 |
| Evidence and audit integrity | covered: B-018。必须有 counters、正负例、coverage、auth 与 exact-head gates。 |
| Cancellation / interruption / partial completion | covered: B-009、B-013。partial stream、取消、断连和 retry 不得伪完成或复用状态。 |

## 边界情况

- 同名工具产生两个不同 call ID：两条 ledger entry 独立，result 只能按 ID 关联。
- assistant message 同时携带文本和多个 calls：文本保持现有拼接语义，calls 保持 part 顺序。
- 两个 assistant turns 在相同 candidate/part 坐标返回无 ID call：fallback ID 必须稳定但跨
  turn 不同；若上游提供非空 ID，则 fallback 不得覆盖。
- `parallel_tool_calls=false`、未知/malformed `tool_choice`、forced 未声明函数，以及 legacy 与
  modern function 字段共存：必须按各自 disposition 映射或 pre-auth/network 拒绝，不得忽略。
- tool result 为 `null`、array、字符串或 error object：均按 B-008 可逆表达，不能变空字符串；
  顶层 Text 固定进入 `result` envelope，顶层 Parts 逐项保留 type/字段/顺序。
- upstream 在多个 chunks 重复发送完整 functionCall：同 ID 只能产生一次身份，重复不变内容可
  幂等，内容冲突必须失败。
- tool call 后 provider fallback 到另一 provider：旧 provider ledger 不得跨边界复用。
- Vertex partner model：不进入 Google tool-loop helper，继续按自身 capability/adapter 处理。
- GH1112 在 implementation 前改变 neutral Google module 边界：先更新本规格 manifest 和锚点，
  不在实现中临时新增另一个 shared owner。

## 发布说明

这是现有 Chat Completions provider 的兼容性修复，不改变用户配置。发布说明应列出 Gemini
Developer/Vertex tool-loop 与 streaming 支持、fail-closed correlation 错误以及认证隔离；
不宣称 #1108 模型刷新、#1112 catalog migration 或 #1113 pricing 已由本 issue 完成。
