# Provider 解耦方案（结合 provider-development 指南）

> 目标：在保持 Trait Object + 统一错误（`ProviderError`）架构前提下，降低 provider 耦合和重复实现。

## 0. 现状问题（与本库对应）

1. Provider 实现模式碎片化（手写 + 多宏 + 多基类并存）。
2. `Provider` enum / dispatch / factory 覆盖不一致。
3. 配置模型不统一（`BaseConfig` / `BaseProviderConfig` / 各 provider 自定义字段风格混杂）。
4. 请求构建、认证、错误映射、流式解析在 provider 内重复。
5. `error.rs`、`error_mapper.rs` 存在多套近似逻辑。

---

## 1. 解耦目标（设计原则）

1. **单一 Provider 抽象面**
   - 所有 provider 实现 `LLMProvider`，错误统一 `ProviderError`。
2. **能力分层而非 provider 堆叠**
   - 认证、请求转换、重试、流式解析拆为可组合组件。
3. **统一构建入口**
   - 一个 provider registry/factory，避免多处 match/dispatch 漂移。
4. **OpenAI-compatible 与非兼容 provider 分治**
   - 兼容类走共享 pipeline，非兼容类仅覆写差异点。

---

## 2. 目标架构（建议）

```text
ProviderRegistry
  -> Box<dyn LLMProvider>

LLMProvider
  - chat_completion
  - chat_completion_stream
  - models/health/cost

ProviderRuntime（可复用）
  - HttpExecutor(GlobalPoolManager)
  - AuthStrategy
  - RequestTransformer
  - ResponseTransformer
  - StreamTransformer
  - HttpErrorMapper(-> ProviderError)
  - RetryPolicy / TimeoutPolicy
```

### 2.1 关键点
- Provider 只声明“差异参数”：base_url 规则、认证方式、模型能力、特殊字段映射。
- 通用 HTTP pipeline（构建、发送、状态处理、错误映射、解析）由 runtime 层托管。

---

## 3. 可执行拆分（按阶段）

## Phase 1：先统一“底座”

1. 选定唯一基础配置（建议 `BaseConfig`）。
2. 选定唯一 header 载体（建议 `HeaderPair`）。
3. 选定唯一 HTTP 错误映射入口（`HttpErrorMapper`）。
4. 选定唯一连接池入口（`GlobalPoolManager`），清理重复 client 创建路径。

**产出**：provider 基础设施只保留一套。

## Phase 2：统一 OpenAI-compatible provider

1. 建立统一 `OpenAICompatibleProviderRuntime`：
   - request transform
   - non-stream execute
   - stream execute + SSE parse
2. 将 OpenAI-compatible provider 迁移为“声明式配置 + hook”：
   - `provider_name`
   - `auth_strategy`
   - `model_catalog`
   - `request/response/stream hooks`

**产出**：大多数 provider 从“实现类”降维为“配置 + 少量差异逻辑”。

## Phase 3：收敛 registry/dispatch/factory

1. 一处 provider 清单生成：enum（如保留）+ dispatch + factory 同源。
2. 禁止手工维护多套 match；通过宏或注册表一次生成。
3. `from_config_async/create_provider` 语义统一，删除永远 NotImplemented 的入口。

**产出**：新增 provider 只改一处注册。

## Phase 4：错误与流式统一

1. 所有 provider 固定 `type Error = ProviderError`。
2. 删除只做别名的 `error.rs`，保留真正有价值的 provider 特殊解析器。
3. 流式统一走 `base/sse` + provider-specific transformer（必要时覆写）。

**产出**：错误模型与流式行为可预测。

---

## 4. 边界设计（防止再次耦合）

1. `core/providers/*` 不再直接持有 Actix/HTTP route 语义。
2. Provider 不直接感知 server 层 DTO；统一使用 core types。
3. Provider 内禁止新增通用工具实现（放入 base/runtime 层）。
4. 新 provider 验收门槛：
   - 统一错误
   - 统一超时/重试
   - 统一流式接口
   - 无重复 header/request pipeline

---

## 5. 迁移顺序建议（低风险）

1. 先选 3 个高度相似 provider（如 exa_ai / featherless / nscale）做模板迁移。
2. 稳定后批量迁移 OpenAI-compatible provider。
3. 最后处理 Anthropic/Bedrock/Vertex 这类非兼容 provider 的差异抽象。

---

## 6. 度量指标（判断是否解耦成功）

1. provider 目录内重复 HTTP 流程代码行数显著下降。
2. `dispatch/factory` 改动点收敛到单一注册源。
3. 新增 provider 必须文件数下降（例如从 5-6 个降到 2-3 个）。
4. provider 相关测试从“复制模板”转为“共享行为测试 + 少量差异测试”。
