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
| 管理端信封 | `src/server/http.rs:57-83, 245-268` | `ApiResponse` 错误一律 400；`to_http_response` 零调用点 | 死代码 + 状态码丢失 |
| 早期失败路径 | `src/server/routes/ai/chat.rs:47-48`、`responses.rs:34-35` | `?` 走 `ResponseError`（副本 2），handler 内走副本 1 | invariant 2 的证据 |
| RequestId 中间件 | `src/server/middleware/request_id.rs:71-88` | 只写响应头，不进错误 body | invariant 4 的落点 |

## 设计方案

1. **canonical 映射模块**：新增 `src/utils/error/http_mapping.rs`（或挂在 `gateway_error/` 下）：

   ```rust
   pub struct ErrorHttpFacts {
       pub status: StatusCode,
       pub openai_type: &'static str,   // OpenAI error.type
       pub code: &'static str,          // canonical code
       pub headers: Vec<(HeaderName, HeaderValue)>, // 预留（#833 填充 Retry-After）
   }
   pub fn http_facts(err: &GatewayError) -> ErrorHttpFacts;
   ```

   现有两张 match 表合并为这一张；`openai_errors.rs` 与 `response.rs` 改为从 `ErrorHttpFacts`
   渲染各自 JSON 形状，不再各自 match。

2. **request_id 注入**：`ResponseError::error_response` 无法拿到中间件扩展——反转依赖：
   `/v1/*` 路由统一改用显式 `gateway_error_response(&err, &ctx)`（`ctx` 携带 request_id），
   extractor 层错误通过 `app_data(web::JsonConfig::default().error_handler(...))` 汇入同一渲染函数。
   `ResponseError` impl 保留为最后兜底（此时 request_id 缺失可接受，但渲染仍走 canonical 表）。

3. **管理端**：`errors::*` helpers（`http.rs:245-268`）改为携带语义状态码构造 `ApiResponse`；
   删除零调用点的 `to_http_response`；信封 JSON 字段不动。

4. **一致性测试**：为 `GatewayError`/`ProviderError` 的代表值集合（每变体至少一个构造样本）
   断言：副本 1 渲染 status == 副本 2 渲染 status == `http_facts().status`。变体新增时测试
   编译期强制覆盖（match 无 `_` 分支）。

5. **Cancelled 决策**（维护者批复项）：建议统一 499（`http_facts` 单点改动即可切换）。

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P1 两路径同 status | http_mapping.rs + 两适配器 | 变体遍历一致性测试 |
| P2 同路由同形状 | chat.rs 等路由 + JsonConfig error_handler | 集成测试：三阶段失败均为 OpenAI 形状 |
| P3 管理端语义状态码 | http.rs errors helpers | keys/teams/budget 路由错误码测试 |
| P4 request_id 一致 | 渲染函数 + RequestIdMiddleware | 集成测试：body request_id == 头 X-Request-ID |
| P5 死代码删除 | http.rs | `rg to_http_response` 零命中 + cargo check |

## 数据流

错误产生（provider/中间件/handler）→ `http_facts()`（唯一决策点）→ 形状适配器
（OpenAI JSON / canonical JSON / 管理端信封）→ HttpResponse（+ request_id 注入）。

## 备选方案

- 只加一致性测试不合并表：漂移会被测试拦住但两张表仍是双倍维护成本，作为过渡可接受、终态拒绝。
- 全部路由改抛 `actix_web::Error` 依赖 `ResponseError`：拿不到 request_id 且形状被路径 2 锁死，拒绝。
- 引入 problem+json（RFC 9457）：破坏 OpenAI 兼容，拒绝。

## 风险

- Security: 无新增面；错误信息脱敏行为保持现状（本 spec 不改 message 内容策略）。
- Compatibility: 管理端状态码变化是对外行为变化，需要 CHANGELOG 矩阵；`/v1/*` 形状统一对
  依赖「早期失败返回 canonical 形状」的客户端是变化（预期无此类客户端，形状本就不稳定）。
- Performance: 每错误一次表查询，可忽略。
- Maintenance: match 无 `_` 分支保证新变体编译期强制归类。

## 测试计划

- [ ] Unit tests: `http_facts` 全变体覆盖；一致性遍历测试。
- [ ] Integration tests: chat 路由三阶段失败形状/状态码/request_id；管理端语义状态码。
- [ ] Manual verification: `curl` 非法 JSON、错误 key、未知模型三类请求对照。

## 回滚方案

单 PR revert；`http_facts` 表与旧两张表在迁移 PR 内并存一个 commit（先加表+测试，再切适配器），
可在中间点回退。
