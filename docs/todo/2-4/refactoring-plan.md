# 代码库重构优化计划 (2024-02-04)

## 概述

本文档记录 litellm-rs 代码库中发现的重复设计和优化机会。

## 问题清单

### 1. HTTP Client 重复创建 [P0]

**问题**: 20+ 处重复的 `Client::builder()` 模式

**位置**:
- `src/core/alerting/channels.rs:43, 178`
- `src/core/guardrails/openai_moderation.rs:27`
- `src/core/fine_tuning/providers/openai.rs:28`
- `src/core/providers/*/provider.rs` (几乎所有 provider)
- `src/core/integrations/observability/*.rs`

**当前代码**:
```rust
let http_client = reqwest::Client::builder()
    .timeout(config.timeout())
    .build()
    .map_err(|e| ...)?;
```

**解决方案**: 创建共享的 HTTP Client 工厂

**预期收益**: 减少 ~500 行重复代码，统一超时/重试配置

---

### 2. Provider 结构高度重复 [P0]

**问题**: 114 个 providers，大部分结构几乎相同

**位置**: `src/core/providers/*/provider.rs`

**重复模式**:
```rust
pub struct XxxProvider {
    config: XxxConfig,
    http_client: reqwest::Client,
    supported_models: Vec<ModelInfo>,
}

impl XxxProvider {
    pub fn new(config: XxxConfig) -> Result<Self, ProviderError> { ... }
    pub async fn with_api_key(api_key: impl Into<String>) -> Result<Self, ProviderError> { ... }
    fn build_headers(&self) -> HashMap<String, String> { ... }
}
```

**解决方案**:
- 方案1: 使用宏 `define_openai_compatible_provider!`
- 方案2: 泛型基类 `OpenAICompatibleProvider<C>`

**预期收益**: 减少 ~5000 行重复代码

---

### 3. Error 类型分散 [P1]

**问题**: 28 个不同的 Error 枚举分布在各处

**位置**:
| 模块 | Error 类型 |
|------|-----------|
| `src/utils/error/` | `GatewayError` |
| `src/core/types/errors/` | `LiteLLMError`, `OpenAIError`, `ConfigError`, `RoutingError` |
| `src/core/traits/` | `CacheError`, `IntegrationError`, `SecretError`, `MiddlewareError` |
| `src/core/providers/` | `ProviderError` |
| `src/core/alerting/` | `AlertError` |
| `src/core/guardrails/` | `GuardrailError` |
| `src/core/a2a/` | `A2AError` |
| `src/core/mcp/` | `McpError` |
| `src/core/realtime/` | `RealtimeError` |
| `src/core/integrations/langfuse/` | `LangfuseError` |
| `src/core/audit/` | `AuditError` |
| `src/core/router/` | `RouterError` |
| `src/core/cost/` | `CostError` |
| `src/auth/oauth/` | `SessionError` |
| `src/sdk/` | `SDKError` |

**解决方案**: 统一错误层次结构，使用 `thiserror` 自动实现 `From`

**预期收益**: 提高可维护性，简化错误处理

---

### 4. Cache 模块重复 [P1]

**问题**: 3 个独立的 cache 相关模块功能重叠

**位置**:
- `src/core/cache/` - 主缓存实现
- `src/core/cache_manager/` - 缓存管理器
- `src/core/semantic_cache/` - 语义缓存

**解决方案**: 合并为单一模块
```
src/core/cache/
├── mod.rs
├── memory.rs
├── redis.rs
├── semantic.rs      # 从 semantic_cache 合并
├── manager.rs       # 从 cache_manager 合并
└── cloud/
```

**预期收益**: 减少混淆，统一缓存 API

---

### 5. Observability/Monitoring 重复 [P2]

**问题**: 3 个独立的监控相关模块

**位置**:
- `src/core/observability/` - 内部 metrics/tracing
- `src/core/integrations/observability/` - 外部集成
- `src/monitoring/` - 又一个监控模块

**解决方案**: 统一为单一模块结构
```
src/core/observability/
├── internal/     # 内部 metrics
├── integrations/ # 外部集成
└── alerts/       # 告警
```

**预期收益**: 减少混淆，统一监控 API

---

### 6. Config Builder 重复模式 [P3]

**问题**: 几乎每个模块都手写相同的 builder 方法

**当前代码**:
```rust
pub fn api_key(mut self, key: impl Into<String>) -> Self {
    self.api_key = Some(key.into());
    self
}
```

**解决方案**: 使用 `typed-builder` 或自定义 derive 宏

**预期收益**: 减少样板代码

---

## 优先级排序

| 优先级 | 任务 | 影响 | 工作量 | 状态 |
|--------|------|------|--------|------|
| P0 | HTTP Client 工厂 | 减少 ~500 行 | 低 | TODO |
| P0 | Provider 宏/泛型 | 减少 ~5000 行 | 高 | TODO |
| P1 | Error 类型统一 | 提高可维护性 | 中 | TODO |
| P1 | Cache 模块合并 | 减少混淆 | 中 | TODO |
| P2 | Observability 合并 | 减少混淆 | 中 | TODO |
| P3 | Config Builder 宏 | 减少样板代码 | 低 | TODO |

## 执行计划

1. **Phase 1**: HTTP Client 工厂 (低风险，快速见效)
2. **Phase 2**: Provider 宏/泛型 (高影响，需要仔细设计)
3. **Phase 3**: Error 类型统一
4. **Phase 4**: Cache 模块合并
5. **Phase 5**: Observability 合并
6. **Phase 6**: Config Builder 宏

---

## 任务跟踪 (更新: 2026-02-04)

### Task 1: HTTP Client 工厂

**目标**: 统一 provider 内的 HTTP client 创建逻辑，保留每个 provider 的 timeout 语义。

**步骤**:
1. 在 `src/core/providers/base/http.rs` 添加 `create_http_client(provider, timeout)` 工厂函数
2. 在 `src/core/providers/base/mod.rs` 导出该函数
3. 替换以下 providers 内的 `Client::builder()` 为工厂函数:
   - `aiml_api`, `bytez`, `aleph_alpha`, `anyscale`, `custom_api`, `compactifai`,
     `comet_api`, `deepl`, `maritalk`, `siliconflow`, `yi`
4. 确认 `timeout` 仍然来自 provider config

**状态**: Done

**进展**:
- 已完成 provider 列表中的 `create_http_client` 替换
- 已扩展到非 provider 模块（见下方 Task 1 扩展）

---

### Task 1 扩展: 非 Provider 模块 Client 复用

**目标**: 将非 provider 模块中的 `Client::builder()` 统一收敛到共享工厂方法（不改变现有错误语义）。

**范围 (本轮)**:
- `src/core/alerting/channels.rs`
- `src/core/guardrails/openai_moderation.rs`
- `src/core/fine_tuning/providers/openai.rs`
- `src/core/integrations/observability/{helicone,arize,datadog,opentelemetry}.rs`
- `src/core/budget/alerts.rs`
- `src/core/webhooks/manager.rs`

**策略**:
- 使用 `crate::utils::net::http::create_custom_client` 替代重复 builder
- 保留原有错误处理路径（`map_err` / `unwrap_or_default` / fallback）

**状态**: Done

**进展**:
- 已用 `create_custom_client` 替换上述模块内的 `Client::builder()`
- 已补充: `auth/oauth`, `sdk/client`, `integrations/langfuse` 的 `Client::builder()` 替换

---

### Task 1 扩展: Provider 模块剩余 Client::builder (简单 timeout 场景)

**目标**: 将仍旧仅设置 timeout 的 provider client 创建，统一迁移到 `create_custom_client`。

**范围 (本轮)**:
- `src/core/providers/v0/mod.rs`
- `src/core/providers/azure/{chat,embed,image}.rs`
- `src/core/providers/azure_ai/{chat,embed,rerank}.rs`
- `src/core/providers/vertex_ai/client.rs`
- `src/core/providers/meta_llama/common_utils.rs`

**策略**:
- 使用 `create_custom_client` 并保持原有错误语义
- 对于有默认 headers 的 provider 使用 `create_custom_client_with_headers`

**进展**:
- 已使用 `create_custom_client` 替换上述 provider 的简单 timeout client
- 新增 `create_custom_client_with_headers` 以支持默认 headers（`azure_ai`, `openrouter`）
- `rg Client::builder` 仅剩 base/shared/macro 内部使用

**状态**: Done

---

### Task 2: OpenAI 兼容 Provider 宏

**目标**: 抽取 OpenAI 兼容 provider 的重复实现，降低样板代码。

**方案**:
- 在 `src/core/providers/macros.rs` 增加 `define_openai_compatible_provider!`
- 支持参数化：provider 名称、config 类型、error mapper、模型列表、默认 base url、鉴权 header、支持参数列表
- 优先迁移完全同构的 provider

**候选迁移列表**:
- `aiml_api`, `anyscale`, `bytez`, `aleph_alpha`, `compactifai`, `comet_api`, `siliconflow`, `yi`

**状态**: Done

**进展**:
- 已新增 `define_openai_compatible_provider!`
- 已迁移: `aiml_api`, `anyscale`, `bytez`, `aleph_alpha`, `compactifai`, `comet_api`, `siliconflow`, `yi`
- 已新增 `define_http_provider_with_hooks!` 支持自定义请求/响应
- 已迁移: `maritalk`, `custom_api`, `deepl`

---

### Task 3: 共享 Client (按 timeout 复用)

**目标**: 统一按 timeout 复用 HTTP client，提升连接池复用率。

**现状**:
- 已存在 `utils::net::http::get_client_with_timeout` 缓存机制

**计划**:
- 优化缓存 key 精度（支持毫秒级 timeout）
- 在宏生成的 provider 中优先使用 `get_client_with_timeout`
- 其他模块后续逐步迁移

**状态**: Done

**进展**:
- `get_client_with_timeout` 缓存 key 改为毫秒级
- 新增 `get_client_with_timeout_fallible` 保留错误语义
- 宏生成的 providers 使用共享 client (按 timeout 复用)

---

### Task 2 扩展: standard_provider! 宏统一 Client 创建

**目标**: `standard_provider!` 宏内部统一使用 `create_custom_client`，避免宏内残留 `Client::builder()`。

**范围 (本轮)**:
- `src/core/providers/macros.rs` (standard_provider)

**状态**: Done

**进展**:
- `standard_provider!` 内部改为 `create_custom_client`，移除 `Client::builder()` 直建

---

### Task 2 扩展: Hook 宏迁移更多 Provider

**目标**: 使用 `define_http_provider_with_hooks!` 迁移仍具备固定 HTTP 模式的 providers。

**进展**:
- `define_http_provider_with_hooks!` 增加自定义 `calculate_cost` 支持
- 已迁移: `openrouter`（保持未知模型 cost 返回 0 的语义）

**状态**: Done

---

### Task 2 附加: 宏变更回归测试

**目标**: 覆盖宏与迁移 provider 的编译与基础行为验证。

**范围 (本轮)**:
- 运行与 provider 宏相关的最小化测试集

**状态**: Done

**进展**:
- `cargo test openrouter` (含 provider/config/models/transformer/typing 相关用例)

---

### Task 2 扩展: Pooled Hook 宏与 Streaming 支持

**目标**: 支持基于 `GlobalPoolManager` 的 provider 模式，并允许自定义 streaming 实现。

**范围 (本轮)**:
- 新增 pooled 版 hook 宏（支持 streaming/自定义 cost）
- 迁移具备相同模式的 provider

**状态**: Done

**进展**:
- 新增 `define_pooled_http_provider_with_hooks!`（支持 GlobalPoolManager + streaming hook）
- 已迁移: `firecrawl`, `empower`
- `cargo test firecrawl` (覆盖 provider tests)

---

### Task 2 扩展: Pooled Hook Provider 迁移 (Batch 2)

**目标**: 继续迁移使用 `GlobalPoolManager` 的 provider 到 pooled hook 宏。

**范围 (本轮)**:
- `ai21`

**状态**: Done

**进展**:
- 已迁移: `ai21`, `amazon_nova`
- `AgentCoordinator` object-safety 已修复，相关测试通过

---

### Task 2 扩展: Pooled Hook Provider 迁移 (Batch 3)

**目标**: 继续迁移使用 `GlobalPoolManager` 的 provider 到 pooled hook 宏。

**范围 (本轮)**:
- `datarobot`
- `deepseek`

**状态**: Done

**进展**:
- 已迁移: `datarobot`, `deepseek`

---

### Task 0: AgentCoordinator Object-Safety 修复

**目标**: 修复 `AgentCoordinator` trait object 测试失败，避免阻塞 provider 测试。

**范围 (本轮)**:
- `src/core/agent/coordinator.rs`

**状态**: Done

**进展**:
- `AgentCoordinator` trait object-safety 已修复
