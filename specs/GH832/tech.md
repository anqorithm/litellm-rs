# Tech Spec

## Linked Issue

GH-832 / #832

## Product Spec

Link to `product.md`.

## Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| Middleware assembly | `src/server/http.rs:198-214` | `.wrap(cors)` 注册最早，Actix 逆序执行后成为最内层 | preflight 先撞 Auth |
| Auth rejection | `src/server/middleware/auth.rs:165-175` | `AuthMethod::None` 返回 401 | preflight 无凭证必失败 |
| Public route helper | `src/server/middleware/helpers.rs:82-97` | 只按 path 判断，不看 method/header | 不能靠 public route 表解决 `/v1/*` preflight |
| CORS config | `src/config/models/gateway.rs` / `src/server/http.rs` | 已有 `CorsConfig` 与 builder | 修复应复用现有配置 |

## 设计方案

1. **CORS 外层化**：调整 `HttpServer` app builder 的 wrap 注册顺序，使 CORS 在 Actix 执行顺序中位于
   auth/rate-limit 之外。当前注释已经说明 wrap 逆序执行；实现必须同步更新注释，避免后来误读。
2. **preflight 判定保护**：如单纯外层化不足以覆盖当前 actix-cors 行为，增加一个小 helper
   `is_cors_preflight(req)`，条件为：
   - method 为 `OPTIONS`；
   - 存在 `Origin`；
   - 存在 `Access-Control-Request-Method`。
   该 helper 只用于避免 auth/rate-limit 误拦标准 preflight，不进入普通 public route 表。
3. **下游响应装饰**：CORS 层必须仍能给 auth/rate-limit 失败响应附加 CORS header，保证浏览器能读取错误。
4. **测试锁顺序**：新增 actix test app 覆盖 preflight 与 unauthenticated POST。测试应从真实
   `configure_routes` / middleware builder 进入，而不是只单测 helper。

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P1 preflight 不鉴权 | http.rs CORS order / helper | OPTIONS + CORS headers returns 2xx/204 |
| P2 非 preflight 不放宽 | AuthMiddleware | POST without auth remains 401 |
| P3 下游错误带 CORS | CORS outer layer | POST 401 response includes allow-origin for allowed origin |
| P4 顺序防回归 | http.rs test/comment | Test fails if CORS moves inside Auth |

## 数据流

浏览器 preflight → outer CORS middleware validates origin/method/headers → CORS responds before auth.

普通 request → outer CORS middleware wraps response → metrics/request-id/auth/rate-limit/routes execute as before
→ response decorated with CORS headers when origin is allowed.

## 备选方案

- 把 `/v1/*` 加入 public route：会绕过真实请求鉴权，拒绝。
- 在 AuthMiddleware 中无条件允许所有 `OPTIONS`：会放宽非 CORS OPTIONS，拒绝。
- 只在浏览器文档中要求带 auth header：浏览器 preflight 本身不会带凭证，拒绝。

## 风险

- Security: 只放行标准 preflight，不放行业务请求；风险低。
- Compatibility: 修复浏览器兼容性；非浏览器路径不变。
- Maintenance: middleware 顺序容易回归，必须用测试锁定。

## 测试计划

- [ ] Integration tests: allowed-origin preflight `/v1/chat/completions`。
- [ ] Integration tests: unauthenticated POST remains 401 and has CORS header.
- [ ] Unit/helper tests: missing `Origin` 或 missing `Access-Control-Request-Method` 不算 preflight。

## 回滚方案

单 PR revert；回滚后浏览器 preflight 再次失败，但不影响非浏览器请求。
