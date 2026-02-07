# 安全性分析报告

## 概述

本报告对 litellm-rs 项目进行全面的安全性审计，涵盖认证/授权、输入验证、敏感信息泄露、CORS 配置、Rate Limiting、TLS 配置、unsafe 代码和时序攻击等维度。

---

## 1. 认证/授权实现安全性

### 1.1 JWT 使用对称密钥算法 (HS256)

- **严重程度**: 中
- **文件**: `src/auth/jwt/handler.rs:19`
- **问题**: JWT 固定使用 `Algorithm::HS256` 对称密钥算法。对称密钥意味着签名和验证使用同一个密钥，如果密钥泄露，攻击者可以伪造任意 token。不支持 RS256/ES256 等非对称算法。

```rust
Ok(Self {
    encoding_key: EncodingKey::from_secret(secret),
    decoding_key: DecodingKey::from_secret(secret),
    algorithm: Algorithm::HS256,  // 硬编码为 HS256
```

- **建议**: 支持配置化的算法选择，生产环境推荐使用 RS256 或 ES256 非对称算法，实现密钥轮换机制。

### 1.2 JWT Token 缺少撤销机制

- **严重程度**: 高
- **文件**: `src/auth/jwt/handler.rs:118-130`
- **问题**: `verify_token` 方法仅验证签名和过期时间，没有检查 token 是否已被撤销。一旦 token 签发，在过期前无法使其失效（例如用户登出后 token 仍然有效）。

```rust
pub async fn verify_token(&self, token: &str) -> Result<Claims> {
    let mut validation = Validation::new(self.algorithm);
    validation.set_issuer(&[&self.issuer]);
    validation.set_audience(&["api", "refresh"]);
    // 没有检查 token 是否在黑名单中
    let token_data = decode::<Claims>(token, &self.decoding_key, &validation)...
```

- **建议**: 实现 token 黑名单机制（基于 Redis），在 `verify_token` 中检查 `jti` 是否已被撤销。登出时将 token 的 `jti` 加入黑名单。

### 1.3 Refresh Token 过期时间计算不合理

- **严重程度**: 中
- **文件**: `src/auth/jwt/handler.rs:74`
- **问题**: Refresh token 的过期时间是 `self.expiration * 24`，这意味着如果 access token 过期时间是 24 小时（默认值 86400 秒），refresh token 将有效 **576 天**（86400 * 24 = 2073600 秒），远超合理范围。

```rust
exp: now + (self.expiration * 24), // Refresh tokens last 24x longer
```

- **建议**: Refresh token 的过期时间应独立配置，建议默认 7-30 天，而非简单乘以 24。

### 1.4 登录响应中 expires_in 硬编码

- **严重程度**: 低
- **文件**: `src/server/routes/auth/login.rs:110`
- **问题**: 登录响应中 `expires_in` 硬编码为 3600，但实际 token 过期时间由配置决定（默认 86400），导致客户端获得错误的过期信息。

```rust
let response = LoginResponse {
    access_token,
    refresh_token,
    token_type: "Bearer".to_string(),
    expires_in: 3600, // 硬编码，与实际配置不一致
```

- **建议**: 从 JWT handler 的 `expiration` 字段获取实际值。

### 1.5 认证可被完全禁用

- **严重程度**: 高
- **文件**: `src/server/middleware/auth.rs:83-88`
- **问题**: 当 `enable_jwt` 和 `enable_api_key` 都为 false 时，所有请求直接跳过认证。虽然有 `warn_insecure_config` 警告，但没有强制阻止。

```rust
let auth_enabled =
    app_state.config.auth().enable_jwt || app_state.config.auth().enable_api_key;
if !auth_enabled {
    req.extensions_mut().insert(context);
    return service.call(req).await;  // 完全跳过认证
}
```

- **建议**: 在非 `dev_mode` 下，禁止同时关闭所有认证方式，或至少在启动时发出严重警告并要求显式确认。

### 1.6 Session Cookie 缺少安全属性

- **严重程度**: 高
- **文件**: `src/server/middleware/helpers.rs:30-38`, `src/server/routes/auth/session.rs:34-41`
- **问题**: Session cookie 的读取仅通过简单的字符串解析 `cookie.strip_prefix("session=")`，但整个代码库中没有任何地方设置 session cookie 的安全属性（HttpOnly、Secure、SameSite）。这使得 session token 容易受到 XSS 攻击窃取和 CSRF 攻击。

```rust
// 仅读取 cookie，没有在任何地方设置安全属性
if let Some(stripped) = cookie.strip_prefix("session=") {
    let session_id = stripped.to_string();
    return AuthMethod::Session(session_id);
}
```

- **建议**: 设置 cookie 时必须包含 `HttpOnly; Secure; SameSite=Strict; Path=/` 属性。

---

## 2. 输入验证

### 2.1 密码重置流程中的用户枚举

- **严重程度**: 中
- **文件**: `src/auth/password.rs:47-56`
- **问题**: `request_password_reset` 方法在用户不存在时返回 `GatewayError::not_found("User not found")`。虽然 HTTP handler 层（`src/server/routes/auth/password.rs:27-29`）正确地对外返回统一响应，但内部日志 `info!("Password reset requested for email: {}", email)` 会记录所有尝试的邮箱地址，可能被用于用户枚举。

```rust
let user = self.storage.db().find_user_by_email(email).await?
    .ok_or_else(|| GatewayError::not_found("User not found"))?;
```

- **建议**: 日志中不应记录完整邮箱地址，应进行脱敏处理（如 `t***@example.com`）。

### 2.2 注册端点泄露用户名/邮箱存在性

- **严重程度**: 中
- **文件**: `src/server/routes/auth/register.rs:45-75`
- **问题**: 注册端点分别返回 "Username already exists" 和 "Email already exists" 的明确错误信息，攻击者可以利用此信息枚举已注册的用户名和邮箱。

```rust
Ok(Some(_)) => {
    return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
        "Username already exists".to_string(),
    )));
}
// ...
Ok(Some(_)) => {
    return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
        "Email already exists".to_string(),
    )));
}
```

- **建议**: 统一返回模糊的错误信息，如 "Registration failed, please try different credentials"。

### 2.3 登录端点泄露用户状态

- **严重程度**: 低
- **文件**: `src/server/routes/auth/login.rs:39-43`
- **问题**: 登录时对非活跃用户返回 "Account is disabled"，这让攻击者知道该用户名确实存在且被禁用。

```rust
if !user.is_active() {
    return Ok(HttpResponse::Forbidden()
        .json(ApiResponse::<()>::error("Account is disabled".to_string())));
}
```

- **建议**: 统一返回 "Invalid credentials"，不区分用户不存在、密码错误、账户禁用等情况。

### 2.4 请求体大小限制但缺少深度验证

- **严重程度**: 低
- **文件**: `src/config/models/server.rs:23`
- **问题**: 虽然配置了 `max_body_size`（默认 10MB），但对 AI 请求中的 `messages` 数组长度、单条消息长度等没有深度限制，可能导致下游 provider 请求超时或资源耗尽。

- **建议**: 对 chat completion 请求中的 messages 数量、单条 message 长度、总 token 数等增加验证。

---

## 3. 敏感信息泄露风险

### 3.1 AuthConfig 中 jwt_secret 可被序列化

- **严重程度**: 高
- **文件**: `src/config/models/auth.rs:10-29`
- **问题**: `AuthConfig` 结构体的 `jwt_secret` 字段同时派生了 `Serialize` 和 `Deserialize`，且没有 `#[serde(skip_serializing)]` 标记。如果配置被序列化输出（如 debug 端点、日志、错误信息），JWT 密钥将被泄露。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,  // 可被序列化输出
```

- **建议**: 为 `jwt_secret` 添加 `#[serde(skip_serializing)]` 属性，并自定义 `Debug` 实现以隐藏密钥值。

### 3.2 认证错误信息泄露内部细节

- **严重程度**: 中
- **文件**: `src/server/middleware/auth.rs:148-153`
- **问题**: 认证失败时，内部错误信息直接返回给客户端，可能泄露系统内部实现细节。

```rust
Err(err) => {
    rate_limiter.record_failure(&client_id);
    Err(actix_web::error::ErrorInternalServerError(format!(
        "Authentication error: {}",
        err  // 内部错误直接暴露给客户端
    )))
}
```

- **建议**: 对外返回通用错误信息 "Internal server error"，将详细错误仅记录到日志。

### 3.3 登录日志记录用户名

- **严重程度**: 低
- **文件**: `src/server/routes/auth/login.rs:16`, `src/server/routes/auth/login.rs:27`
- **问题**: 登录尝试（包括失败的）会记录完整用户名到日志中。在高频暴力破解场景下，日志可能包含大量敏感信息。

```rust
info!("User login attempt: {}", request.username);
warn!("Login attempt with invalid username: {}", request.username);
warn!("Login attempt with invalid password for user: {}", request.username);
```

- **建议**: 对用户名进行脱敏处理后再记录日志。

### 3.4 TokenPair 的 Debug 实现泄露 token 值

- **严重程度**: 中
- **文件**: `src/auth/jwt/types.rs:78`
- **问题**: `TokenPair` 结构体派生了 `Debug`，其 `access_token` 和 `refresh_token` 字段会在 debug 输出中完整显示。相比之下，`JwtHandler` 正确地自定义了 Debug 实现来隐藏密钥。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,   // Debug 时完整输出
    pub refresh_token: String,  // Debug 时完整输出
```

- **建议**: 自定义 `Debug` 实现，对 token 值进行截断或脱敏。

---

## 4. CORS 配置

### 4.1 默认 CORS 允许所有来源

- **严重程度**: 高
- **文件**: `src/config/models/server.rs:184-195`
- **问题**: `CorsConfig` 默认 `allowed_origins` 为空数组，而 `allows_all_origins()` 方法将空数组视为允许所有来源。这意味着默认配置下 CORS 对所有来源开放。

```rust
impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_origins: vec![], // 空 = 允许所有来源
            // ...
        }
    }
}

pub fn allows_all_origins(&self) -> bool {
    self.allowed_origins.is_empty() || self.allowed_origins.contains(&"*".to_string())
}
```

- **建议**: 默认应拒绝所有跨域请求，要求用户显式配置允许的来源。空数组应表示"不允许任何跨域请求"。

### 4.2 示例配置使用通配符

- **严重程度**: 中
- **文件**: `config/gateway.yaml.example:26-28`
- **问题**: 示例配置文件中 CORS 使用 `allowed_origins: ["*"]` 和 `allowed_headers: ["*"]`，用户直接复制使用会导致安全风险。

```yaml
cors:
    allowed_origins: ["*"]
    allowed_headers: ["*"]
```

- **建议**: 示例配置应使用具体的域名示例，并添加注释说明通配符的安全风险。

### 4.3 CORS 中间件实现为空壳

- **严重程度**: 中
- **文件**: `src/server/middleware/security.rs:50-57`
- **问题**: `CorsMiddlewareService` 的 `call` 方法是空实现，直接透传请求，没有实际执行任何 CORS 检查。实际的 CORS 处理依赖 `actix-cors` crate（在 `server.rs` 中配置），但自定义的 `CorsMiddleware` 可能误导开发者认为已有 CORS 保护。

```rust
fn call(&self, req: ServiceRequest) -> Self::Future {
    let fut = self.service.call(req);
    Box::pin(async move {
        let res = fut.await?;
        Ok(res)  // 没有任何 CORS 处理
    })
}
```

- **建议**: 删除空壳的 `CorsMiddleware`，避免混淆。确保 `actix-cors` 的配置覆盖所有路由。

---

## 5. Rate Limiting

### 5.1 Rate Limiting 中间件未实现

- **严重程度**: 高
- **文件**: `src/server/middleware/rate_limit.rs:69-102`
- **问题**: `RateLimitMiddlewareService` 的 `call` 方法中，rate limiting 逻辑被注释为 "Rate limiting logic would go here"，实际上只是记录日志然后直接放行所有请求。

```rust
fn call(&self, req: ServiceRequest) -> Self::Future {
    // ...
    Box::pin(async move {
        if let Some(_state) = &app_state {
            // Rate limiting logic would go here
            // For now, just log and pass through
            debug!("Rate limit check for {} {} - start: {:?}", method, path, start_time);
        }
        let res = fut.await?;
        Ok(res)
    })
}
```

- **建议**: 实现实际的 rate limiting 逻辑，使用 token bucket 或 sliding window 算法，基于配置的 `RateLimitConfig` 进行限流。

### 5.2 Rate Limiting 默认禁用

- **严重程度**: 中
- **文件**: `src/config/models/rate_limit.rs:23-31`
- **问题**: `RateLimitConfig` 默认 `enabled: false`，且即使启用，中间件也未实现（见 5.1）。

```rust
impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: false,  // 默认禁用
```

- **建议**: 生产环境应默认启用 rate limiting，并提供合理的默认值。

### 5.3 认证 Rate Limiter 使用内存存储

- **严重程度**: 中
- **文件**: `src/server/middleware/auth_rate_limiter.rs:9-20`
- **问题**: `AuthRateLimiter` 使用 `DashMap` 内存存储，在多实例部署时无法共享限流状态。攻击者可以对不同实例分别发起暴力破解攻击。

```rust
pub struct AuthRateLimiter {
    attempts: DashMap<String, AuthAttemptTracker>,  // 仅内存存储
```

- **建议**: 支持 Redis 作为分布式限流后端，确保多实例间共享限流状态。

### 5.4 认证 Rate Limiter 无内存上限

- **严重程度**: 中
- **文件**: `src/server/middleware/auth_rate_limiter.rs:39`
- **问题**: `DashMap` 没有容量限制，攻击者可以使用大量不同的 IP/API key 组合来填充内存，导致 OOM。虽然有 `cleanup_old_entries` 方法，但没有自动调用机制。

- **建议**: 设置 DashMap 的最大容量，实现定期自动清理（如通过 tokio 定时任务），或使用 LRU 缓存替代。

---

## 6. TLS 配置

### 6.1 TLS 默认未启用

- **严重程度**: 高
- **文件**: `src/config/models/server.rs:28`, `src/server/server.rs:166-169`
- **问题**: TLS 默认为 `None`（未启用），且服务器启动时使用 `bind`（HTTP）而非 `bind_rustls`/`bind_openssl`（HTTPS）。即使配置了 TLS，`start` 方法中也没有使用 TLS 配置。

```rust
// server.rs - 始终使用 HTTP bind
let server = ActixHttpServer::new(move || Self::create_app(state.clone()))
    .bind(&bind_addr)  // 没有 TLS
```

- **建议**: 当 TLS 配置存在时，使用 `bind_rustls` 或 `bind_openssl` 启动 HTTPS 服务器。生产环境应强制要求 TLS。

### 6.2 TLS 配置缺少最低版本和密码套件控制

- **严重程度**: 中
- **文件**: `src/config/models/server.rs:115-126`
- **问题**: `TlsConfig` 仅包含证书和密钥路径，没有配置最低 TLS 版本（应至少 TLS 1.2）和密码套件选择。

```rust
pub struct TlsConfig {
    pub cert_file: String,
    pub key_file: String,
    pub ca_file: Option<String>,
    pub require_client_cert: bool,
    // 缺少: min_tls_version, cipher_suites
}
```

- **建议**: 添加 `min_tls_version`（默认 TLS 1.2）和 `cipher_suites` 配置项。

### 6.3 HSTS 头在非 TLS 模式下也被设置

- **严重程度**: 低
- **文件**: `src/server/middleware/security.rs:119-122`
- **问题**: `SecurityHeadersMiddleware` 无条件设置 `Strict-Transport-Security` 头，即使服务器运行在 HTTP 模式下。这在纯 HTTP 环境中没有意义，且可能在反向代理场景下造成混淆。

```rust
headers.insert(
    HeaderName::from_static("strict-transport-security"),
    HeaderValue::from_static("max-age=31536000; includeSubDomains"),
);
```

- **建议**: 仅在 TLS 启用时设置 HSTS 头。

### 6.4 缺少 Content-Security-Policy 头

- **严重程度**: 中
- **文件**: `src/server/middleware/security.rs:100-131`
- **问题**: `SecurityHeadersMiddleware` 设置了 X-Content-Type-Options、X-Frame-Options、X-XSS-Protection、HSTS、Referrer-Policy，但缺少 `Content-Security-Policy` 头。

- **建议**: 添加 `Content-Security-Policy: default-src 'none'` 头（对于 API 网关来说，这是合理的默认值）。

---

## 7. Unsafe 代码

### 7.1 env::set_var 在多线程环境中使用 unsafe

- **严重程度**: 中
- **文件**: `src/utils/logging/utils/utils.rs:33-35`, `src/utils/config/utils.rs:50-53`
- **问题**: 多处使用 `unsafe { env::set_var(...) }`。在 Rust 2024 edition 中 `env::set_var` 被标记为 unsafe，因为在多线程环境中修改环境变量是未定义行为。该项目是异步多线程应用（Tokio + Actix），这些调用可能导致数据竞争。

```rust
// utils/logging/utils/utils.rs:33
pub fn set_verbose(enabled: bool) {
    unsafe {
        env::set_var("LITELLM_VERBOSE", if enabled { "true" } else { "false" });
    }
}

// utils/config/utils.rs:51
pub fn set_env_var(key: &str, value: &str) {
    unsafe {
        env::set_var(key, value);
    }
}
```

- **建议**: 使用 `std::sync::OnceLock` 或 `AtomicBool` 替代环境变量来存储运行时配置。如果必须使用环境变量，应仅在程序启动的单线程阶段设置。

### 7.2 get_unchecked_mut 在 Stream 实现中

- **严重程度**: 低
- **文件**: `src/core/traits/transformer.rs:70`
- **问题**: 使用 `unsafe { self.get_unchecked_mut() }` 来获取 Pin 内部的可变引用。虽然在 `S: Unpin` 约束下这是安全的（因为 Unpin 类型可以安全地移动），但使用 unsafe 增加了维护风险。

```rust
fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    let this = unsafe { self.get_unchecked_mut() };
    Pin::new(&mut this.stream).poll_next(cx)
}
```

- **建议**: 由于 `S: Unpin`，可以安全地使用 `Pin::get_mut(self)` 替代 unsafe 代码。

---

## 8. 时序攻击风险

### 8.1 constant_time_eq 在长度不等时提前返回

- **严重程度**: 中
- **文件**: `src/utils/auth/crypto/hmac.rs:26-37`
- **问题**: `constant_time_eq` 函数在字符串长度不同时立即返回 `false`，这泄露了长度信息。虽然对于 HMAC 签名验证（固定长度的 hex 字符串）影响较小，但作为通用的常量时间比较函数，这是一个缺陷。

```rust
pub(crate) fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;  // 长度不等时提前返回，泄露长度信息
    }
    let mut result = 0u8;
    for (a_byte, b_byte) in a.bytes().zip(b.bytes()) {
        result |= a_byte ^ b_byte;
    }
    result == 0
}
```

- **建议**: 使用 `subtle` crate 的 `ConstantTimeEq` trait，它提供了经过审计的常量时间比较实现。或者在长度不等时仍然执行完整比较（使用较长字符串的长度）。

### 8.2 API Key 验证通过数据库查询进行

- **严重程度**: 低
- **文件**: `src/auth/api_key/creation.rs:103-116`
- **问题**: API key 验证先计算 SHA256 哈希，然后通过数据库查询匹配。数据库查询的时间差异可能泄露哈希是否存在。不过由于先进行了哈希运算，攻击者无法直接利用时序差异推断原始 key。

```rust
pub async fn verify_key(&self, raw_key: &str) -> Result<Option<(ApiKey, Option<User>)>> {
    let key_hash = hash_api_key(raw_key);
    let api_key = match self.storage.db().find_api_key_by_hash(&key_hash).await? {
        Some(key) => key,
        None => { return Ok(None); }  // 未找到时提前返回
    };
```

- **建议**: 风险较低，但可以考虑在未找到 key 时添加随机延迟，使响应时间更一致。

---

## 9. 其他安全问题

### 9.1 SSRF 防护存在 DNS 重绑定风险

- **严重程度**: 中
- **文件**: `src/config/validation/ssrf.rs:117-136`
- **问题**: SSRF 防护在验证时解析 DNS，但实际请求时可能再次解析 DNS。如果攻击者控制 DNS 服务器，可以在验证和请求之间更改 DNS 记录（DNS 重绑定攻击），使验证时解析到公网 IP，实际请求时解析到内网 IP。

```rust
if !host_is_literal {
    let port = url.port_or_known_default().unwrap_or(80);
    if let Ok(addrs) = (host, port).to_socket_addrs() {
        for addr in addrs {
            if is_private_or_internal_ip(&addr.ip()) {
                return Err(...);
            }
        }
    }
}
```

- **建议**: 在实际发起 HTTP 请求时也验证解析后的 IP 地址，或使用自定义 DNS 解析器将验证和请求绑定到同一次 DNS 解析结果。

### 9.2 API Key 使用 SHA256 而非 HMAC 进行哈希

- **严重程度**: 低
- **文件**: `src/utils/auth/crypto/keys.rs:45-49`
- **问题**: API key 使用纯 SHA256 哈希存储，没有加盐。虽然 API key 本身是高熵随机值（32 字符字母数字），彩虹表攻击不太现实，但加盐是更好的实践。

```rust
pub fn hash_api_key(api_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    hex::encode(hasher.finalize())
}
```

- **建议**: 使用 HMAC-SHA256 并配合服务端密钥，或使用 bcrypt/argon2 进行哈希（但会影响查询性能）。

### 9.3 Server 头泄露服务器信息

- **严重程度**: 低
- **文件**: `src/server/server.rs:149`
- **问题**: 响应中设置了 `Server: LiteLLM-RS` 头，泄露了服务器软件信息，有助于攻击者进行针对性攻击。

```rust
.wrap(DefaultHeaders::new().add(("Server", "LiteLLM-RS")))
```

- **建议**: 移除或替换为通用值，或使其可配置。

### 9.4 密码重置 Token 使用 Alphanumeric 而非 CSPRNG

- **严重程度**: 低
- **文件**: `src/utils/auth/crypto/keys.rs:29-35`
- **问题**: `generate_token` 使用 `rand::thread_rng()` 的 `Alphanumeric` 分布生成 token。虽然 `thread_rng()` 在大多数平台上使用 CSPRNG，但 `Alphanumeric` 限制了字符集（62 个字符），降低了每字符的熵。32 字符的 token 约有 190 bit 熵，对于密码重置来说足够，但不是最优。

- **建议**: 使用 `OsRng` 生成随机字节后进行 base64url 编码，以获得更高的每字符熵。

---

## 严重程度汇总

| 严重程度 | 数量 | 关键问题 |
|---------|------|---------|
| **高** | 5 | Rate Limiting 未实现、TLS 未启用、CORS 默认全开放、JWT 无撤销机制、Session Cookie 无安全属性 |
| **中** | 12 | JWT 对称密钥、Refresh Token 过期时间、用户枚举、AuthConfig 序列化泄露、unsafe env::set_var 等 |
| **低** | 7 | 登录状态泄露、HSTS 非 TLS 设置、Server 头泄露、API Key 哈希无盐等 |

## 优先修复建议

1. **P0 (立即修复)**: 实现 Rate Limiting 中间件、启用 TLS 支持、修复 CORS 默认配置
2. **P1 (短期修复)**: 实现 JWT token 撤销机制、修复 Refresh Token 过期时间、为 AuthConfig 添加序列化保护、设置 Session Cookie 安全属性
3. **P2 (中期改进)**: 替换 unsafe env::set_var、使用 `subtle` crate 进行常量时间比较、统一错误信息防止用户枚举、添加 CSP 头
4. **P3 (长期优化)**: 支持非对称 JWT 算法、分布式 Rate Limiting、DNS 重绑定防护、API Key 加盐哈希



