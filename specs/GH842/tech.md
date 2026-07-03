# Tech Spec

## Linked Issue

GH-842 / #842

## Product Spec

Link to `product.md`.

## Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| chat stream | `src/server/routes/ai/chat.rs:83-127` | `request_for_budget = request.clone()`，构造 `core_request` 后 closure 内再 clone `core_request`、`context`、`request_for_budget` | stream 热路径重复 clone 大请求体 |
| chat non-stream | `src/server/routes/ai/chat.rs:396-455` | `request_for_budget` 与 `request_for_cache` 各 clone 一份，retry closure 内再 clone request/context/handles | 非 stream 热路径重复 clone |
| Auth context | `src/server/middleware/auth.rs:415-448` | 每个请求复制全部非鉴权 header 到 `HashMap<String,String>` | 大 header 集下每请求固定分配 |
| Context extraction | `src/server/routes/ai/context.rs:28-32` | 从 request extensions 取 `RequestContext` 后 clone 整体返回 | handler 捕获再次复制 context |
| RequestContext 类型 | `src/core/types/context.rs:12-33` | `#[derive(Clone)]`，headers/metadata 为 owned `HashMap` | 需要共享或瘦身 |
| AppState KeyManager | `src/server/state.rs:45-46` | `key_manager: KeyManager` 按值存放；chat 等 spend 站点 clone manager | clone 会复制 `hmac_secret: Option<String>` |
| KeyManager | `src/core/keys/manager.rs:24-33` | repository/cache 为 `Arc`，但 `hmac_secret` 是 owned `Option<String>` | 可改为 `Option<Arc<str>>` 或把 AppState 字段改为 `Arc<KeyManager>` |

## 设计方案

1. **共享 chat request**
   - 在 `handle_chat_completion_internal` 与 `handle_streaming_chat_completion` 入口尽早将原始 `ChatCompletionRequest`
     包成 `Arc<ChatCompletionRequest>`。
   - response cache、budget estimation、token policy 读取原始 request 时接收 `&ChatCompletionRequest` 或 `Arc` 引用，
     不再需要 `request_for_budget` / `request_for_cache` 两份 owned clone。
   - `build_core_chat_request` 拆成借用版本（例如 `build_core_chat_request_ref(&ChatCompletionRequest, ...)`）或只 clone最终 provider request 所需字段。
   - `prepare_chat_request_for_provider` 改为接收 borrowed original request + owned/borrowed core template，并只在生成最终 provider request 时拥有必要数据。

2. **stream usage 注入保持隔离**
   - 当前 stream 会把 `stream_options.include_usage` 强制设为 true。迁移后不要 mut 原始 `Arc<ChatCompletionRequest>`。
   - 建议新增小型 builder：从 borrowed original request 构造 provider request 时注入内部 usage 需求，同时保留 `client_requested_usage` 用于响应过滤。
   - budget/cache 读取必须使用未污染的 original request。

3. **共享 RequestContext**
   - 在 request extensions 中存放 `Arc<RequestContext>`，并让 `get_request_context` 返回 `Arc<RequestContext>` 或新的 `RequestContextRef`。
   - handler 与 retry closure clone 的是 `Arc`，不是 `HashMap` / metadata。
   - `RequestContext` 本身的 public builder API 可保留；middleware 构建完成后再 `Arc::new(context)`。
   - header 收集改为白名单或 lazy 模式：只保留实际使用字段（request id、user agent、client ip、auth metadata），自定义 headers 仅在有明确调用点时填充。鉴权 header 继续排除。

4. **共享 KeyManager**
   - 首选把 `AppState.key_manager` 改为 `Arc<KeyManager>`，保持 manager 内部状态共享，AI route closure 只 clone `Arc`。
   - 若调用面过大，也可以先将 `KeyManager.hmac_secret` 改为 `Option<Arc<str>>`，并在第二步收敛 `AppState` 字段；但最终验收需证明 AI route 不复制 secret string。
   - `/v1/keys` 管理路由必须继续通过同一个 manager 访问 repository 和 last-used throttle cache。

5. **可测 guard**
   - 为 chat 增加大 payload 聚焦测试或 criterion benchmark，构造多 message + image_url/base64 payload。
   - 如 Rust 代码难以直接统计 clone 次数，可用 allocation counter、custom test payload size、或 benchmark 输出作为证据。
   - 添加静态 guard：在 `chat.rs` 中禁止 `request.clone()` 用于 budget/cache 分叉，允许注释说明的最终 provider request clone。

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P1 行为不变 | chat stream/non-stream | 现有 chat tests + budget/cache 聚焦测试 |
| P2 请求共享 | `chat.rs` + token_policy/cache helpers | 静态 guard + 大 payload allocation/bench 对比 |
| P3 context 共享 | auth middleware + `context.rs` + handlers | 单测：metadata/auth fields 可见；handler closure clone 不复制 HashMap |
| P4 header 安全 | auth middleware | 单测：`authorization` / `x-api-key` 不进入 context |
| P5 KeyManager 共享 | `state.rs` + key routes + spend call sites | 单测/compile：`Arc<KeyManager>` 或 `Arc<str>` secret；key route tests 通过 |

## 风险

- Compatibility: `get_request_context` 返回类型变化会触碰多个 route；迁移需一次性修完编译错误，不能留 Any/alias 逃逸。
- Security: header 收集调整不能泄漏鉴权 header，也不能丢失 auth metadata。
- Performance: `Arc` 降低 clone 成本，但如果 builder 为每次 attempt 仍 clone完整 messages，收益会被抵消；provider request 构造需单独检查。

## 测试计划

- [ ] `cargo test server::routes::ai::chat --lib --all-features`
- [ ] `cargo test server::middleware::auth --lib --all-features`
- [ ] `cargo test core::keys --lib --all-features`
- [ ] 大 payload allocation test 或 criterion bench，PR body 记录迁移前后数据。
- [ ] `cargo test --all-features`
- [ ] 静态 guard: `rg -n "request_for_budget|request_for_cache|request\\.clone\\(\\)|context_for_execution = context\\.clone\\(\\)|state\\.key_manager\\.clone\\(\\)" src/server/routes/ai/chat.rs src/server/routes/ai`.

## 回滚方案

优先按 chat/request sharing、context sharing、KeyManager sharing 三个提交分层实现；任一层出问题可单独 revert。公开 API schema 不变，回滚不需要数据迁移。
