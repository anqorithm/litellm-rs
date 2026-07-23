# Product Spec

## Linked Issue

GH-1112 / #1112

- `complexity: large`
- `spec_approval: user_approved`
- `approval_source: 2026-07-22 current conversation ("批准 ... 使用implxauto模式")`

## 用户问题

Google Gemini 的模型事实目前分散在三处：Gemini provider registry、
`VertexAIModel` 枚举/解析器，以及 Vertex provider 的静态 `models()` 列表。三处对同一
模型的生命周期、上下文限制、能力和请求参数可以互相矛盾；Vertex 还会用 substring
模糊匹配把非精确 ID 归类为已知模型，并把其他字符串包装成 `Custom`。

用户需要一个共享、可审计、精确匹配的 Google 模型事实源。Gemini Developer API 与
Vertex AI 应复用模型核心事实和请求契约，同时仍以各自明确的 availability、endpoint
与认证 overlay 决定能否调用。目录复用不得把 Developer API 的可用性推断到 Vertex，
也不得把 Gemini API key 与 Vertex Bearer token 混入同一认证路径。

## 目标

- Google Gemini 模型的稳定 ID、生命周期、核心能力、限制和请求契约只有一个事实源。
- Gemini Developer API 与 Vertex AI 通过显式 availability overlay 公开各自模型目录。
- provider 的模型声明、请求校验、supported params 和请求转换消费同一份精确模型契约。
- 消除 Vertex 的 substring 模型识别与未知 `Custom` chat model 绕过路径。
- 模型列表稳定排序、无重复，退役或无对应入口证据的模型不得被误报为可调用。
- 保持 Gemini Developer API query API key 与 Vertex `Authorization: Bearer` 认证隔离。

## 非目标

- 不在本 issue 中刷新 #1108 的 2026-07 Developer API 模型 ID、价格或 live smoke。
- 不实现 #1111 的 `ToolUse` / `ToolResult` 与 `functionCall` / `functionResponse` 回路。
- 不修复 #1113 的重复定价、未知模型零成本或 spend/budget parity。
- 不统一 Gemini 与 Vertex 的 HTTP client、endpoint builder、credential 类型或重试实现。
- 不改变 Vertex partner-model、embedding、image、batch 或 model-garden 的产品范围。
- 不新增自动模型发现、后台同步任务、远端缓存或第二套 provider registry。

## Behavior Invariants

1. **B-001** 每个共享 Google chat model 必须以一个区分大小写的 exact model ID 作为
   canonical key；lookup 不得使用 substring、大小写折叠、前后缀猜测或未声明 alias。
2. **B-002** 同一 canonical key 的生命周期、核心能力、上下文/输出限制和请求契约
   只有一个定义；Gemini 与 Vertex consumer 不得复制这些字段为独立常量或 match 表。
3. **B-003** Developer API 与 Vertex availability 必须分别显式声明。模型只在对应
   overlay 为 available 且有该入口的来源证据时才出现在该 provider 的 `models()`；
   一侧 available 不得隐式推出另一侧 available。
4. **B-004** provider 返回的模型列表必须按 canonical ID 稳定升序、无重复；相同输入
   多次构建目录所得顺序和内容完全一致，不依赖 `HashMap` 迭代顺序。
5. **B-005** retired、unavailable、未证实或仅限其他 Google 产品的模型不得被公开为
   当前通用 chat model；对它们的请求在网络调用前返回 typed model-not-found/
   unsupported-model 错误。
6. **B-006** Vertex chat model 解析只接受 catalog 或显式 partner catalog 中的 exact
   ID。包含已知 ID 的额外前缀/后缀、大小写变体、空字符串和未知字符串均不得映射为
   已知模型。
7. **B-007** 未知 Vertex chat model 不得通过 `Custom(String)` 获得默认 capability、
   context limit、supported params、URL 路由或请求执行；配置 custom `api_base` 也不改变
   exact-model 校验结果。
8. **B-008** 每个公开模型的请求契约必须明确列出允许参数以及与模型相关的约束；
   `get_supported_openai_params`、preflight validation 和实际 request body 必须由同一
   契约导出，不能一处声明支持而另一处透传或拒绝。
9. **B-009** 缺失请求契约、请求参数不在 allowlist、参数值越界或模型特定 illegal
   state 时必须在网络前 fail closed；不得 warning 后删除、保留原值或套用另一模型的
   默认契约继续请求。
10. **B-010** 对同时在 Developer API 和 Vertex available 的 exact model ID，相同的
    provider-neutral 请求字段必须得到相同的允许/拒绝结论；endpoint-specific 字段必须
    由 overlay 明确区分，不能污染共享核心契约。
11. **B-011** Gemini Developer API 请求继续只通过 Developer credential 路径携带
    API key；共享 catalog、Vertex overlay、错误文本和 debug/display 输出均不得持有、
    复制或记录该 key。
12. **B-012** Vertex 请求继续由 `VertexAuth` 获取 token 并使用
    `Authorization: Bearer`；不得从 Gemini API-key 字段生成 Bearer token，也不得把
    Vertex token 放入 query string、catalog 或模型 metadata。
13. **B-013** 共享 catalog 只决定模型事实和 provider-neutral 请求契约；Gemini 与
    Vertex 的 base URL、publisher、region、API version、auth header/query 和 transport
    保持各 provider 所有，目录查询不得触发 credential 获取或网络访问。
14. **B-014** 显式、已发布的兼容 alias 若未来存在，必须在 catalog 中单独声明并解析
    到唯一 canonical ID，且不能扩大任一 provider availability；本 issue 不以模糊匹配
    自动生成 alias。
15. **B-015** catalog 初始化必须拒绝 duplicate ID、alias collision、缺失 lifecycle、
    缺失 request contract、available 但无来源证据等非法记录；非法 catalog 不得降级为
    空列表或部分列表后启动成功。
16. **B-016** 并发读模型目录与请求契约必须只观察到同一不可变快照；初始化完成后
    目录不得按请求修改，重复/并发查询不得产生顺序漂移或部分状态。
17. **B-017** 迁移后现有已证实且仍 available 的 exact model ID 保持兼容；
    `vertex_ai::models::VertexAIModel` 与 `parse_vertex_model` 的公开导入路径继续指向唯一
    canonical Vertex 类型和 exact `Result` parser，不得以 alias/wrapper 保留旧
    fuzzy/`Custom` chat 语义；删除或停止广告的 ID 必须由 lifecycle/availability fixture
    解释，不能因重构意外消失。
18. **B-018** 回归证据必须同时覆盖正例与 schema/类型合法的负例：exact lookup、
    fuzzy rejection、unknown/custom rejection、双 overlay 差异、stable ordering、
    request-contract parity、网络请求计数为零以及 production credential 类型的
    Debug/Display/log redaction；测试不能只避免格式化 credential 来绕过 redaction gate。

## 验收标准

- [ ] 只有一个共享 Google model catalog 定义核心 metadata、availability overlay 和
      request contract；Gemini/Vertex 不再各自维护 Gemini chat model 表。
- [ ] Gemini 和 Vertex `models()` 均由共享 catalog 过滤并稳定排序，fixture 证明无重复。
- [ ] Vertex parser 对 exact IDs 成功；已知 ID 的前后缀、大小写变体、空值与未知值均
      在网络前失败，`Custom` 不再是 chat model fallback。
- [ ] 两个 provider 的 supported params、validation 和 request transformation 对重叠
      模型遵循同一共享 contract，负例证明不会 silent drop 或 silent passthrough。
- [ ] Developer-only、Vertex-only、retired 与 unverified fixture 分别得到正确公开/拒绝
      结果，任一 availability 不从另一入口推断。
- [ ] 无网络认证测试证明 Developer 请求只使用 query API key、Vertex 请求只使用
      Bearer token；`GeminiConfig`、`VertexCredentials`、`ServiceAccountKey`、
      `AuthorizedUserCredentials` 的 Debug/Display/log 捕获和 catalog/error 均不出现测试凭证。
- [ ] #1108、#1111、#1113 的独立验收面没有被本实现顺带修改或宣称完成。
- [ ] 新增关键 catalog/validation 分支覆盖 100%，新增代码总体 line coverage 至少 80%。
- [ ] `cargo fmt --check`、`cargo check`、strict Clippy、`cargo test` 与 SpecRail gates 通过。

## 边界检查

| 边界类别 | 判定 |
| --- | --- |
| Empty / missing input | covered: B-006、B-009、B-015。空 model、缺失 contract 和非法记录均 fail closed。 |
| Error and failure paths | covered: B-005、B-007、B-009、B-015。未知、退役、非法参数和非法 catalog 都有显式错误。 |
| Authorization / permission | covered: B-011、B-012、B-013。两类 credential 和 endpoint owner 保持隔离。 |
| Concurrency / race / ordering | covered: B-004、B-016。不可变快照与稳定排序消除并发/迭代顺序漂移。 |
| Retry / repetition / idempotency | covered: B-004、B-016。重复查询幂等；校验失败发生在网络前，因此不进入 retry。 |
| Illegal state transitions | covered: B-003、B-005、B-015。无证据记录不能从 unavailable 变成 advertised。 |
| Compatibility / migration | covered: B-014、B-017。只保留显式 alias，并用 fixture 解释目录差异。 |
| Degradation / fallback | covered: B-005、B-007、B-009。不允许 Custom、默认 contract 或部分 catalog 伪装成功。 |
| Evidence and audit integrity | covered: B-003、B-015、B-018。availability 必须带来源证据，负例必须到达业务 gate。 |
| Cancellation / interruption / partial completion | N/A：catalog 是进程内同步初始化和不可变只读查询；请求取消/stream lifecycle 不在本 issue 范围。 |

## 边界情况

- `foo-gemini-2.5-pro`、`gemini-2.5-pro-preview-extra` 和大小写变体不能命中
  `gemini-2.5-pro`。
- 一个 ID 在 Developer available、Vertex unavailable 时，只出现在 Gemini 列表；反向
  同理。
- catalog 同时出现 canonical ID 与同名 alias 时初始化失败，而不是 last-write-wins。
- 自定义 Vertex base URL 与 public base URL 使用同一 exact-model gate。
- partner model 继续走独立 exact partner catalog；不得因本次 Google catalog 重构退回
  substring 分类。
- pricing consumer 可以按 canonical ID 读取现有价格数据，但 pricing authority 的收敛
  和未知价格语义仍由 #1113 决定。

## 发布说明

这是行为收紧的内部目录重构：模型列表顺序将稳定，未证实、退役、模糊或未知 ID 会在
网络前被拒绝。发布说明应列出被停止广告的 ID 及证据，并说明 Gemini Developer API
与 Vertex availability、endpoint 和认证仍然独立。不得把本变更描述为 #1108 的模型
刷新、#1111 的工具回路修复或 #1113 的 pricing 修复。
