# Product Spec

## Linked Issue

GH-839 / #839

## 用户问题

在 `origin/main@c47596a4`，同一个错误可能以三种不同的 HTTP 形状/状态码返回给客户端，取决于它在
哪个阶段失败、走了哪条代码路径：

1. OpenAI 形状 `{error:{message,type,param,code}}`（`src/server/routes/ai/openai_errors.rs:126-324`）；
2. `ResponseError` 形状 `{error:{code,canonical_code,retryable,...}}`（`src/utils/error/gateway_error/response.rs:10-191`）；
3. 管理端 `ApiResponse` 信封 `{success,data,error,meta}`，错误帮助函数在
   `src/server/routes/mod.rs:18-83,233-268` 生成，部分路径会丢失语义状态码。

已经发生的漂移：`ProviderError::Cancelled` 在路径 1 是 400、在路径 2 是 499。`/v1/chat/completions`
的早期失败（extractor / `?`）走路径 2，handler 内失败走路径 1——同一路由两种 JSON 形状。错误响应的
`request_id` 恒为 null。对客户端 SDK 来说，错误处理与重试逻辑无法可靠编写。

## 目标

- 状态码与错误码词表映射只在一处定义，两个适配器（OpenAI 形状、canonical 形状）消费同一张表。
- OpenAI-compatible AI 端点的任何失败阶段返回同一 JSON 形状；管理端 `/v1/*` 路由继续使用
  `ApiResponse` 信封。
- 管理端错误保留真实语义状态码（404/409/500 不再统一 400）。
- 错误响应携带与中间件分配一致的 `request_id`。

## 非目标

- 不改变 OpenAI-compatible AI 端点对外错误 JSON 的字段命名（保持 OpenAI 兼容）。
- 不补全 #833 范围内缺失的 429 `Retry-After` 策略；但本 spec 必须保留现有
  provider/gateway rate-limit 头（`Retry-After`、`X-RateLimit-*`），并为 #833 预留 headers 位。
- 不重新设计 `GatewayError`/`ProviderError` 的变体集合（#715 已完成的职责拆分保持不变）。

## Behavior Invariants

1. 任意 `(GatewayError | ProviderError)` 值经路径 1 与路径 2 产生的 HTTP status 一致（属性测试可枚举断言）。
2. OpenAI-compatible AI 路由在 extractor 失败（JSON/query/path）、中间件拒绝（auth/rate-limit）、
   handler 内失败三个阶段返回的 JSON 形状相同（OpenAI 形状）。
3. 管理端点（keys/teams/budget/auth）即使挂在 `/v1/*` 下，也继续使用 `ApiResponse` 信封；
   NotFound → 404、冲突 → 409、内部错误 → 500；`{success,data,error}` 信封字段保持兼容。
4. 错误响应 body 中的 `request_id` 与响应头 `X-Request-ID` 一致（可关联日志）。
5. `ApiResponse::to_http_response` 是经 `server::routes` 暴露的 public API：默认保留并标记
   deprecated；若删除，必须在 spec approval 中明确批准 public API break 并写入发布说明。

## 验收标准

- [ ] 单一 canonical 映射模块 `(error) -> (status, openai_code, canonical_code, legacy_code, headers)`
      存在，两个适配器均调用它。
- [ ] 映射事实同时保留 OpenAI `error.code` 词表与 canonical/internal `code` / `canonical_code`
      词表，不把两者压成一个字段。
- [ ] 现有 429 头透传不回退：provider `Retry-After` 与 gateway `Retry-After` / `X-RateLimit-*`
      在统一映射后仍出现在响应中。
- [ ] `ProviderError::ApiError` 中上游 OpenAI `message` / `type` / `param` / `code` 字段在
      OpenAI 适配器中保持原样透传。
- [ ] 映射一致性测试：遍历错误变体断言两路径 status 相等（含 `Cancelled` 的漂移修复决策：400 或 499 二选一，见 tech spec）。
- [ ] AI 路由三阶段失败形状一致的集成测试，覆盖 `JsonConfig`、`QueryConfig`、`PathConfig`
      extractor 失败以及 auth/rate-limit 中间件拒绝。
- [ ] 管理端错误状态码语义化 + 兼容性测试（信封字段不变）。
- [ ] 错误 body `request_id` 非 null 且与头一致。

## 边界情况

- 流式（SSE）已开始后才失败：错误只能以 SSE event 传递，形状对齐 OpenAI 流式错误事件，不在本 spec 的 HTTP 状态码矩阵内，但错误码取值必须来自同一张表。
- actix extractor 层错误：JSON 反序列化失败需要 `JsonConfig`，query/path 反序列化失败需要
  `QueryConfig` / `PathConfig`，否则会绕过统一渲染。
- 中间件拒绝：`AuthMiddleware` 与 `RateLimitError` 发生在 handler 之前，AI 路由必须仍能进入
  OpenAI 形状渲染并注入同一个 request_id。
- `Cancelled` 语义决策：客户端主动断开 → 499（nginx 惯例）更准确；对外 OpenAI 兼容层无 499 先例 → 需要维护者拍板（默认建议 499 内部记录、对外 400 保持 OpenAI 兼容不成立时统一 499）。

## 发布说明

管理端错误状态码从部分丢失语义变为稳定语义化 4xx/5xx，属于对外行为变化；OpenAI-compatible
AI 路由错误形状统一到 OpenAI 形状，管理端 `/v1/*` 继续使用 `ApiResponse` 信封。若删除
`ApiResponse::to_http_response`，CHANGELOG 必须列为 public API break；默认方案是保留并
deprecated。CHANGELOG 需列出状态码变化矩阵。
