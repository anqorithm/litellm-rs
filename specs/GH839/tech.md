# Tech Spec

## Linked Issue

GH-839 / #839

## Product Spec

Link to `product.md`.

## Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| OpenAI 形状映射 | `src/server/routes/ai/openai_errors.rs:126-324` | ~200 行枚举全部错误变体 → (status, type, code) | 待收敛副本 1 |
| Canonical 形状映射 | `src/utils/error/gateway_error/response.rs:10-191` | ~180 行再枚举一遍 → (status, code, message)；`request_id: None` 硬编码（`:203`） | 待收敛副本 2 |
| 已知漂移 | `openai_errors.rs:311-316` vs `response.rs:114-119` | `ProviderError::Cancelled` 400 vs 499 | 一致性测试样本 |
| 管理端信封 | `src/server/routes/mod.rs:18-83,233-268` | `ApiResponse` 与 `errors::*` helpers 位于 routes 模块；`to_http_response` 是 public 方法但仓库内零调用点 | 状态码语义化 + public API 兼容 |
| 早期失败路径 | `src/server/routes/ai/chat.rs:47-48`、`responses.rs:34-35` | `?` 走 `ResponseError`（副本 2），handler 内走副本 1 | invariant 2 的证据 |
| RequestId 中间件 | `src/server/middleware/request_id.rs:71-88` | 只写响应头，不进错误 body | invariant 4 的落点 |
| Extractor 配置 | `src/server/routes/ai/batches.rs:57`、`responses/lifecycle.rs:123` 等 | AI 路由除 JSON 外还使用 `web::Query` / `web::Path` | 需要 `QueryConfig` / `PathConfig` 进入统一渲染 |
| Auth / rate-limit 中间件 | `src/server/middleware/auth.rs:113-174`、`rate_limit.rs:325-346` | handler 前直接构造 actix 错误或 429 JSON | bad key / rate-limit 需要同一 request_id 与 AI 错误形状 |

## 设计方案

1. **canonical 映射模块**：新增 `src/utils/error/http_mapping.rs`（或挂在 `gateway_error/` 下）：

   ```rust
   pub struct ErrorHttpFacts {
       pub status: StatusCode,
       pub openai_type: &'static str,   // OpenAI error.type
       pub openai_code: &'static str,   // OpenAI error.code，保持 lower-case OpenAI 词表
       pub canonical_code: &'static str, // canonical_code，保持内部稳定词表
       pub legacy_code: &'static str,   // canonical JSON 中现有 error.code（如 VALIDATION_ERROR）
       pub headers: Vec<(HeaderName, HeaderValue)>, // 当前动态 rate-limit 头 + #833 扩展点
   }
   pub fn http_facts(err: &GatewayError) -> ErrorHttpFacts;
   ```

   现有两张 match 表合并为这一张；`openai_errors.rs` 与 `response.rs` 改为从 `ErrorHttpFacts`
   渲染各自 JSON 形状，不再各自 match。`headers` 必须从错误实例保留当前已存在的动态头：
   provider `Retry-After`，以及 gateway `Retry-After` / `X-RateLimit-Limit-Requests` /
   `X-RateLimit-Limit-Tokens`。#833 只负责补全更完整的 Retry-After 策略，不允许本 issue
   回退现有头。

   OpenAI 适配器还需要保留 `ProviderError::ApiError` 的 per-instance 覆盖：如果上游 body
   已是 OpenAI `{error:{message,type,param,code}}`，则 OpenAI JSON 的这四个字段沿用上游值；
   canonical/internal JSON 仍使用 `canonical_code` / `legacy_code` 与本地 message 策略。

2. **request_id 注入**：`ResponseError::error_response` 无法拿到中间件扩展——反转依赖：
   OpenAI-compatible AI 路由统一改用显式 `gateway_error_response(&err, &ctx)`（`ctx` 携带
   request_id）。extractor 层错误通过 `app_data(web::JsonConfig::default().error_handler(...))`、
   `web::QueryConfig::default().error_handler(...)`、`web::PathConfig::default().error_handler(...)`
   汇入同一渲染函数。`ResponseError` impl 保留为最后兜底（此时 request_id 缺失可接受，但
   渲染仍走 canonical 表）。

   中间件拒绝也必须进入统一计划：`AuthMiddleware` 的 missing/bad credentials 与
   `RateLimitError::error_response` 应构造可映射错误并按请求所属路由族渲染。AI 路由使用
   OpenAI 形状并注入 request_id；管理端路由继续使用 `ApiResponse` 信封。

3. **管理端**：`src/server/routes/mod.rs:233-268` 的 `errors::*` helpers 改为携带语义状态码
   构造 `ApiResponse`。`ApiResponse::to_http_response` 是 public API，默认保留并加
   `#[deprecated]` 或改为调用语义状态码 helper；只有维护者明确批准 breaking change 时才删除。
   信封 JSON 字段不动。

4. **一致性测试**：为 `GatewayError`/`ProviderError` 的代表值集合（每变体至少一个构造样本）
   断言：副本 1 渲染 status == 副本 2 渲染 status == `http_facts().status`。变体新增时测试
   编译期强制覆盖（match 无 `_` 分支）。

5. **Cancelled 决策**（维护者批复项）：建议统一 499（`http_facts` 单点改动即可切换）。

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P1 两路径同 status | http_mapping.rs + 两适配器 | 变体遍历一致性测试 |
| P2 AI 路由同形状 | chat.rs 等 AI 路由 + JsonConfig/QueryConfig/PathConfig + middleware render path | 集成测试：extractor / auth-rate-limit / handler 失败均为 OpenAI 形状 |
| P3 管理端语义状态码 | routes/mod.rs errors helpers + 管理端路由 | keys/teams/budget 路由错误码测试，信封字段保持 |
| P4 request_id 一致 | 渲染函数 + RequestIdMiddleware | 集成测试：body request_id == 头 X-Request-ID |
| P5 public API 处理 | routes/mod.rs `ApiResponse::to_http_response` | 保留/deprecated 的编译测试；若删除则 release note + human gate |

## 数据流

错误产生（provider/extractor/中间件/handler）→ `http_facts()`（唯一决策点，含分离的
OpenAI 与 canonical code 词表、动态 headers）→ 按路由族选择形状适配器（OpenAI JSON /
canonical JSON / 管理端信封）→ HttpResponse（+ request_id 注入）。

## 备选方案

- 只加一致性测试不合并表：漂移会被测试拦住但两张表仍是双倍维护成本，作为过渡可接受、终态拒绝。
- 全部路由改抛 `actix_web::Error` 依赖 `ResponseError`：拿不到 request_id 且形状被路径 2 锁死，拒绝。
- 引入 problem+json（RFC 9457）：破坏 OpenAI 兼容，拒绝。

## 风险

- Security: 无新增面；错误信息脱敏行为保持现状（本 spec 不改 message 内容策略）。
- Compatibility: 管理端状态码变化是对外行为变化，需要 CHANGELOG 矩阵；OpenAI-compatible
  AI 路由形状统一对依赖「早期失败返回 canonical 形状」的客户端是变化（预期无此类客户端，
  形状本就不稳定）。`ApiResponse::to_http_response` 删除属于 public API break，默认避免。
  OpenAI `error.code` 与 canonical/internal code 词表必须分离，避免静默改变客户端判断逻辑。
- Performance: 每错误一次表查询，可忽略。
- Maintenance: match 无 `_` 分支保证新变体编译期强制归类。

## 测试计划

- [ ] Unit tests: `http_facts` 全变体覆盖；一致性遍历测试；OpenAI vs canonical code 词表测试；
      rate-limit headers 保留测试；上游 OpenAI `ApiError` 字段透传测试。
- [ ] Integration tests: AI 路由 extractor（JSON/query/path）/ middleware（auth/rate-limit）/
      handler 失败形状、状态码、request_id；管理端语义状态码与信封字段。
- [ ] Manual verification: `curl` 非法 JSON、坏 key、rate-limit、未知模型、管理端 404 五类请求对照。

## 回滚方案

单 PR revert；`http_facts` 表与旧两张表在迁移 PR 内并存一个 commit（先加表+测试，再切适配器），
可在中间点回退。
