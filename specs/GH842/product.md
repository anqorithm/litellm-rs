# Product Spec

## Linked Issue

GH-842 / #842

## 用户问题

chat completion 热路径在 stream 与非 stream 分支中多次整包 clone `ChatCompletionRequest`。请求体可能包含长 messages、
tool definitions、甚至 base64 图片；每次 clone 都会放大延迟和内存峰值。认证中间件还会把全部非鉴权 header
复制进 `RequestContext.headers`，随后 handler 再 clone 整个 `RequestContext`。`AppState.key_manager` 按值存放，
AI route 为 spend 记录反复 clone，连带 `hmac_secret: Option<String>` 分配。

## 目标

- chat stream 与非 stream 路径不再多次整包 clone `ChatCompletionRequest`。
- `RequestContext` 在 middleware 与 handler 之间共享，避免每个 handler 再深拷贝 headers / metadata。
- `KeyManager` 在 `AppState` 中按共享 handle 存放或内部 secret 共享化，避免 spend 调用点复制 HMAC secret。
- 请求处理语义、缓存命中语义、预算/计费语义和 provider 调用结果保持不变。

## 非目标

- 不改变 OpenAI-compatible API schema。
- 不重构非 chat 端点，除非为共享 `RequestContext` / `KeyManager` 类型调整所必需。
- 不改变 `allowed_models`、`max_tokens_per_request`、response cache、token policy 或预算结算策略。
- 不在本 issue 中解决 #840 的预算编排抽象。

## Behavior Invariants

1. 非 stream chat 与 stream chat 在相同输入下选择相同 provider/model，执行相同预算检查、预留、结算和 response cache 行为。
2. 大请求体只在必须拥有独立可变 provider request 时 clone；budget、cache、token policy、执行闭包共享同一个请求视图。
3. `RequestContext` 的 `api_key_id`、`api_key_budget_id`、`api_key_max_tokens_per_request`、user/team metadata、request id、client ip、user agent 在 handler 中可见值不变。
4. 认证敏感 header 仍不得进入可下游透传的 context；迁移不能重新泄漏 `authorization` 或 `x-api-key`。
5. `KeyManager` 行为不变：API key hash、verify、last_used throttle cache 和 `/v1/keys` 管理路由仍共享同一 manager 状态。

## 验收标准

- [ ] `src/server/routes/ai/chat.rs` stream 与非 stream 路径不再 clone 完整 `ChatCompletionRequest` 3-4 次。
- [ ] `RequestContext` 从 middleware extensions 到 handler 捕获使用共享 handle，chat handler 不再为 retry closure 深拷贝 headers / metadata。
- [ ] `AppState.key_manager` 或 `KeyManager.hmac_secret` 改为共享表示，AI route spend 调用点不再复制 HMAC secret 字符串。
- [ ] 大 payload 聚焦测试或 benchmark 证明请求 clone/alloc 明显下降，PR body 附对比数据。
- [ ] `cargo test --all-features` 通过。

## 边界情况

- `stream_options.include_usage` 当前会被强制设为 true 以便内部计费，再按客户端请求过滤输出；迁移不能改变这个行为。
- `prepare_chat_request_for_provider` 仍需要根据 selected provider/model 和 key token policy 生成 provider-specific request；允许这一步为最终 provider 请求拥有一份必要副本。
- response cache lookup 必须看到原始非 stream request，不得被 token policy 或 stream usage 注入污染。
- `RequestContext` 可能被测试或 legacy helper 直接构造；公共构造方法需要保持源码兼容或提供清晰迁移。

## 发布说明

内部性能优化，无公开 API 变化。CHANGELOG 以 `perf(chat)` 记录。
