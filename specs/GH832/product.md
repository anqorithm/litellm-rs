# Product Spec

## Linked Issue

GH-832 / #832

## 用户问题

浏览器跨域调用 `/v1/*` 时，CORS preflight `OPTIONS` 请求不带 `Authorization`。当前
`src/server/http.rs` 先注册 `.wrap(cors)`，但 Actix wrap 逆序执行，导致 CORS 位于最内层；
请求先经过 `AuthMiddleware`，在 `src/server/middleware/auth.rs:165-175` 因 `AuthMethod::None`
返回 401，CORS 没机会生成 `Access-Control-Allow-*` 响应头。

结果是浏览器客户端无法使用 gateway，即使 CORS 配置允许该 origin。

## 目标

- CORS preflight 由 CORS 层处理，不要求 API key / JWT。
- CORS header 出现在 preflight 与后续 auth/rate-limit 失败响应上。
- 非 preflight 请求的鉴权、限流、metrics 行为不被放宽。

## 非目标

- 不改变 CORS 配置语义或默认 allowed origin。
- 不把任意 `OPTIONS` 请求都变成公开业务路由；只处理标准 CORS preflight。
- 不调整 RateLimit/Auth 的相对顺序，除非为 CORS 外层化所必需。

## Behavior Invariants

1. 符合 CORS preflight 条件的请求（`OPTIONS` + `Origin` + `Access-Control-Request-Method`）不进入
   `AuthMiddleware` 的 missing-auth 401 分支。
2. 非 preflight 的 `OPTIONS` 或其他方法仍按原有 auth/rate-limit 规则处理。
3. CORS middleware 必须处于能装饰所有下游响应的位置；auth 失败、rate-limit 失败也应带允许的
   CORS 响应头。
4. 变更后 `RequestIdMiddleware`、`MetricsMiddleware`、`RateLimitMiddleware`、`AuthMiddleware`
   的业务请求相对语义保持不变。

## 验收标准

- [ ] 集成测试：允许 origin 的 preflight `/v1/chat/completions` 返回 2xx/204，并带
      `Access-Control-Allow-Origin`、`Access-Control-Allow-Methods`、`Access-Control-Allow-Headers`。
- [ ] 集成测试：同一路径不带 auth 的真实 `POST` 仍返回 401，但响应包含 CORS header。
- [ ] 集成测试：没有 `Origin` 或没有 `Access-Control-Request-Method` 的 `OPTIONS` 不被误判为 preflight。
- [ ] middleware 注册顺序有注释或测试锁定，防止再次把 CORS 放回最内层。

## 边界情况

- CORS disabled 时不新增 preflight 放行。
- 不允许的 origin 仍由 CORS 配置拒绝或不返回 allow header。
- 管理端和健康检查路由不因本修复改变鉴权规则。

## 发布说明

修复浏览器客户端无法通过 CORS preflight 的 bug。非浏览器客户端和非 preflight 请求行为不变。
