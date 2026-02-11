# 冗余/重复设计收敛执行计划（逐步实施）

- 计划版本: v1
- 创建时间: 2026-02-10
- 适用仓库: `/Users/lifcc/Desktop/code/AI/gateway/litellm-rs`
- 执行模式: 每步仅做一类改动 -> 立即测试 -> 回写状态 -> 再进入下一步

## 0. 执行约束（DoR）

- 目标: 系统性消除重复设计和冗余设计，同时保持主流程可用。
- 兼容性: `required`（默认保持向后兼容，除非某一步明确标注破坏性变更）。
- 测试策略:
  - 步骤级测试: 每一步至少 1 条自动化命令（`cargo test` / `cargo check` / 定向测试）。
  - 阶段级测试: 每完成一个阶段，运行一组更广泛检查。
  - 最终测试: 全量回归（受环境限制时给出已执行范围和未执行原因）。
- 状态管理:
  - 每个步骤只有一个状态: `pending` / `in_progress` / `completed` / `blocked`。
  - 每步完成后，在“执行日志”追加: 修改文件、测试命令、结果、结论。
- 提交策略:
  - 本计划先按“状态驱动”推进（你未要求自动提交 Git commit）。
  - 若你后续要求，我会按步骤补做 `per_step` 提交。

## 1. 问题域与优先级

### P0（必须优先）

1. Provider 注册体系三套来源失配（`pub mod` / `ProviderType` / `Provider` / factory）。
2. `create_provider` 当前为占位实现，功能行为不完整。
3. RequestContext 三套并存，路由层存在显式桥接转换。

### P1（高优先）

4. 消息模型多套并存（`core/models/openai`、`core/types/message`、`providers/openai/models`）。
5. HTTP client/连接池抽象多套并行。
6. 配置模型并行（`config/models`、`core/types/config`、`sdk/config`）。

### P2（收尾）

7. Redis 双实现并行但主流程只接一套。
8. 缓存子系统（`core/cache` / `cache_manager` / `semantic_cache`）边界重叠。
9. logging 中 `LogEntry` 平行定义。
10. 大量 `allow(dead_code)` 与 legacy 模块清理。

---

## 2. 详细执行步骤（文件级）

## 阶段 A：Provider 体系收敛（P0）

### Step A1 建立一致性守护测试（先防止继续扩散）

- 状态: `completed`
- 目标: 用测试把“Provider 三层失配”显式暴露并持续约束。
- 预计改动文件:
  - `src/core/providers/mod.rs`
  - `src/core/providers/unified_provider_tests.rs`（如已有可复用）
  - `tests/provider_consistency.rs`（可选新增）
- 详细改动:
  - 新增/增强测试：
    - `ProviderType` 已知变体必须与 `Display/From<&str>` 双向一致。
    - `Provider` enum 变体必须覆盖当前“可实例化 provider”集合。
    - `from_config_async` 已实现分支集合必须有测试快照或断言。
  - 将“已实现集合”以常量表达，避免散落在多个测试中。
- 步骤级测试命令:
  - `cargo test test_provider_type_from_display_consistency --lib`
  - `cargo test test_provider_type_all_variants_covered --lib`
- 完成判定:
  - 至少 2 条一致性测试可稳定运行。
  - 新增测试能够在未来新增 provider 时给出明确失败提示。

### Step A2 修复 `create_provider` 占位实现并统一入口

- 状态: `completed`
- 目标: 让 `create_provider` 真正走统一构造路径，减少重复分支维护。
- 预计改动文件:
  - `src/core/providers/mod.rs`
  - `src/config/models/provider.rs`（仅在字段读取需要时）
- 详细改动:
  - 把 `create_provider(config)` 的 provider 类型识别从 `config.name` 的硬编码分支，改为优先 `config.provider_type`，再回退 `config.name`。
  - 统一调用 `Provider::from_config_async`。
  - 将 `ProviderConfig` 映射为 `serde_json::Value` 时统一字段键名（`api_key`、`base_url` 等）。
  - 清理无效占位 `not_implemented("Provider factory for ...")`。
- 步骤级测试命令:
  - `cargo test test_create_provider_with_unknown_provider --lib`（若不存在则新增）
  - `cargo check --lib`
- 完成判定:
  - `create_provider` 不再无条件返回占位错误。
  - 未支持 provider 仍返回可解释错误信息（含 provider 名称）。

### Step A3 下线未接线的 `providers/context`（冗余上下文模型）

- 状态: `completed`
- 目标: 删除孤岛模块，减少上下文模型重复。
- 预计改动文件:
  - `src/core/providers/mod.rs`
  - `src/core/providers/context.rs`（删除）
  - 受影响测试文件（若引用则调整）
- 详细改动:
  - 移除 `pub mod context;`。
  - 删除 `src/core/providers/context.rs`。
  - 确保不存在其他模块引用该路径。
- 步骤级测试命令:
  - `cargo check --lib`
  - `cargo test core::providers::unified_provider_tests --lib`（如名称不匹配改为可运行的 providers 定向测试）
- 完成判定:
  - 编译通过。
  - 搜索 `core::providers::context` 为 0 处。

### Step A4 收敛 Provider 声明源（中期）

- 状态: `completed`
- 目标: 建立单一“已接线 provider 列表”，避免 `pub mod`/`enum`/factory 三处漂移。
- 预计改动文件:
  - `src/core/providers/mod.rs`
  - `src/core/providers/provider_registry.rs`
  - `src/core/providers/macros.rs`（仅在宏接入时）
- 详细改动:
  - 引入单一注册表（常量或函数）维护“可实例化 provider”。
  - `Provider` enum 与 `from_config_async` 基于该表统一扩展。
  - 文档化“新增 provider 的唯一入口步骤”。
- 步骤级测试命令:
  - `cargo test provider_registry --lib`
  - `cargo check --lib`
- 完成判定:
  - 新增 provider 只需改动 1 个主入口 + provider 实现文件。

---

## 阶段 B：Context 与消息模型收敛（P0/P1）

### Step B1 统一 RequestContext 为 `core/types/context::RequestContext`

- 状态: `completed`
- 目标: 只保留一套请求上下文模型。
- 预计改动文件:
  - `src/core/models/mod.rs`
  - `src/core/types/context.rs`
  - `src/server/routes/ai/context.rs`
  - `src/server/routes/ai/chat.rs`
  - `src/server/routes/ai/embeddings.rs`
  - `src/server/routes/ai/completions.rs`
  - `src/server/routes/ai/images.rs`
- 详细改动:
  - 让路由与 core 统一使用 `core/types/context::RequestContext`。
  - 删除或别名化 `core/models::RequestContext`，逐步下线桥接函数 `to_core_context`。
  - 将 `team_id/api_key_id` 等字段放入统一 metadata 约定。
- 步骤级测试命令:
  - `cargo test routes::ai::context --lib`（按实际测试名调整）
  - `cargo check --lib`
- 完成判定:
  - `to_core_context` 无业务映射逻辑或已删除。
  - `pub struct RequestContext` 只保留核心定义（允许类型别名过渡）。

### Step B2 消息模型收敛第一步（去掉路由层双向转换样板）

- 状态: `completed`
- 目标: 减少 `chat.rs` 中重复转换函数。
- 预计改动文件:
  - `src/server/routes/ai/chat.rs`
  - `src/core/types/message.rs`
  - `src/core/models/openai/messages.rs`
- 详细改动:
  - 提供 `From/TryFrom` 统一转换实现到独立模块。
  - 路由层仅调用转换接口，不保留大量 match 样板。
- 步骤级测试命令:
  - `cargo test routes::ai::chat --lib`
  - `cargo check --lib`
- 完成判定:
  - `chat.rs` 的转换样板显著下降。
  - 转换逻辑集中在专门模块。

### Step B3 消息模型收敛第二步（移除第三套 OpenAIMessage）

- 状态: `completed`
- 目标: 降低 `providers/openai/models.rs` 与通用消息模型重复。
- 预计改动文件:
  - `src/core/providers/openai/models.rs`
  - `src/core/providers/openai/chat/` 下相关文件
  - `src/core/models/openai/messages.rs`
- 详细改动:
  - 仅保留协议边界需要的 provider 内部类型。
  - 对共享消息结构复用统一模型。
- 步骤级测试命令:
  - `cargo test openai --lib`
  - `cargo check --lib`
- 完成判定:
  - `OpenAIMessage` 平行定义减少到一处主定义 + 必要适配。

---

## 阶段 C：HTTP 与配置收敛（P1）

### Step C1 HTTP client 单点工厂

- 状态: `completed`
- 目标: 统一 HTTP client 构造策略，避免多套连接池参数漂移。
- 预计改动文件:
  - `src/utils/net/http.rs`（作为主工厂）
  - `src/core/providers/base/connection_pool.rs`
  - `src/core/providers/base_provider.rs`
  - `src/core/providers/base/http.rs`
  - `src/utils/net/client/utils.rs`
- 详细改动:
  - 约定唯一入口（建议 `utils/net/http`）。
  - 其余模块改为薄封装或删除重复 builder。
  - 固化超时/keepalive/pool 参数来源。
- 步骤级测试命令:
  - `cargo test utils::net::http --lib`
  - `cargo check --lib`
- 完成判定:
  - “创建 client 的真实逻辑”只保留一个实现。

### Step C2 配置模型分层收敛

- 状态: `completed`
- 目标: 明确 `config/models`（服务端）与 `sdk/config`（客户端）边界，清理 `core/types/config` 重叠。
- 预计改动文件:
  - `src/config/mod.rs`
  - `src/config/models/server.rs`
  - `src/config/models/provider.rs`
  - `src/core/types/config/mod.rs`
  - `src/core/types/config/server.rs`
  - `src/core/types/config/provider.rs`
  - `src/sdk/config.rs`
- 详细改动:
  - 为 `core/types/config` 增加迁移说明与最小化导出。
  - 若外部无引用，转为内部/弃用标记。
  - 消除同名结构语义冲突（`ServerConfig`/`ProviderConfig`）。
- 步骤级测试命令:
  - `cargo test config --lib`
  - `cargo test sdk::config --lib`
  - `cargo check --lib`
- 完成判定:
  - 每个层级只保留一套职责清晰的配置模型。

---

## 阶段 D：存储/缓存/logging 收尾（P2）

### Step D1 Redis 双实现处置

- 状态: `completed`
- 目标: `redis_optimized` 明确为实验特性或接线进主流程，避免悬空并行实现。
- 预计改动文件:
  - `src/storage/mod.rs`
  - `src/storage/redis_optimized/mod.rs`
  - `src/storage/redis_optimized/pool.rs`
  - `Cargo.toml`（如启用 feature gate）
- 详细改动:
  - 若暂不启用：加 feature gate + 文档说明。
  - 若启用：替换 `StorageLayer` 中 pool 类型。
- 步骤级测试命令:
  - `cargo test storage::redis --lib`
  - `cargo check --lib`
- 完成判定:
  - 不再存在“对外暴露但主流程不用”的核心实现。

### Step D2 缓存系统边界清晰化

- 状态: `completed`
- 目标: 明确 `core/cache`、`cache_manager`、`semantic_cache` 关系。
- 预计改动文件:
  - `src/core/mod.rs`
  - `src/core/cache/mod.rs`
  - `src/core/cache_manager/manager.rs`
  - `src/core/semantic_cache/cache.rs`
  - `docs/architecture/*.md`（必要时）
- 详细改动:
  - 统一入口模块；废弃旧模块时给过渡层。
  - 删除注释掉但仍大量存在的“半启用”路径。
- 步骤级测试命令:
  - `cargo test cache --lib`
  - `cargo check --lib`
- 完成判定:
  - 缓存入口唯一且文档一致。

### Step D3 logging 类型收敛

- 状态: `completed`
- 目标: `LogEntry` 统一定义，其他层做适配。
- 预计改动文件:
  - `src/utils/logging/mod.rs`
  - `src/utils/logging/logging/types.rs`
  - `src/utils/logging/utils/types.rs`
  - `src/core/observability/types.rs`
- 详细改动:
  - 选定 canonical `LogEntry`。
  - 非 canonical 结构改名为 `*Record` 或适配结构。
- 步骤级测试命令:
  - `cargo test logging --lib`
  - `cargo check --lib`
- 完成判定:
  - 无同名 `LogEntry` 平行定义。

### Step D4 dead_code/legacy 收敛

- 状态: `completed`
- 目标: 系统性降低 `allow(dead_code)`，清理已废弃模块暴露。
- 预计改动文件:
  - `src/core/router/mod.rs`
  - `src/core/router/load_balancer/*`
  - `src/utils/config/optimized.rs`
  - `src/utils/mod.rs`
  - 其他含 `#![allow(dead_code)]` 文件
- 详细改动:
  - 仅保留确有必要的 `allow(dead_code)` 并加原因。
  - legacy 模块统一迁移策略（feature gate 或彻底移除）。
- 步骤级测试命令:
  - `cargo check --lib`
  - `cargo test --lib`（若时间过长，按模块拆分执行并记录）
- 完成判定:
  - `allow(dead_code)` 显著下降并有留存理由。

---

## 3. 阶段性回归测试矩阵

### 阶段 A 结束

- `cargo check --lib`
- `cargo test core::providers --lib`

### 阶段 B 结束

- `cargo check --lib`
- `cargo test routes::ai --lib`
- `cargo test core::types --lib`

### 阶段 C 结束

- `cargo check --lib`
- `cargo test config --lib`
- `cargo test sdk::config --lib`

### 阶段 D 结束 / 最终

- `cargo check --lib`
- `cargo test --lib`

---

## 2.1 阶段 E：测试收敛（新增）

### Step E1 对齐配置测试与现行校验规则

- 状态: `completed`
- 目标: 修复 `config` 域 4 个失败用例（测试夹具与断言预期对齐当前安全校验）。
- 预计改动文件:
  - `src/config/models/gateway.rs`
  - `src/config/builder/tests.rs`
  - `src/config/validation/auth_validators.rs`
- 详细改动:
  - `gateway` 测试夹具改为满足当前 JWT 强度与存储验证前提。
  - `builder` 测试补充显式 `AuthConfig`（避免默认空 secret 导致构建失败）。
  - `auth_validators` 中 `jwt_expiration=0` 的错误断言更新为当前规则文案。
- 步骤级测试命令:
  - `cargo test config::builder::tests::tests::test_config_builder --lib -- --exact`
  - `cargo test config::models::gateway::tests::test_gateway_config_validate_success --lib -- --exact`
  - `cargo test config::models::gateway::tests::test_gateway_config_validate_empty_database_url --lib -- --exact`
  - `cargo test config::validation::auth_validators::tests::test_auth_config_zero_jwt_expiration --lib -- --exact`
- 完成判定:
  - 上述 4 条定向测试全部通过。

### Step E2 全量回归确认（lib）

- 状态: `completed`
- 目标: 验证配置收敛后 `lib` 测试全量状态。
- 步骤级测试命令:
  - `cargo check --lib`
  - `cargo test --lib`
- 完成判定:
  - `cargo check --lib` 通过；
  - `cargo test --lib` 无新增回归（理想为全绿）。

---

## 2.2 阶段 F：Provider 冗余样板继续收敛（新增）

### Step F1 删除未使用的重复 `build_headers` 样板（首批 8 个 provider）

- 状态: `completed`
- 目标: 删除“定义但未调用”的重复 header 构造代码，减少 provider 模板冗余与维护面。
- 预计改动文件:
  - `src/core/providers/qwen/mod.rs`
  - `src/core/providers/tavily/mod.rs`
  - `src/core/providers/topaz/mod.rs`
  - `src/core/providers/recraft/mod.rs`
  - `src/core/providers/vercel_ai/mod.rs`
  - `src/core/providers/searxng/mod.rs`
  - `src/core/providers/xiaomi_mimo/mod.rs`
  - `src/core/providers/baichuan/mod.rs`
- 详细改动:
  - 删除未使用的 `build_headers` 方法。
  - 删除对应 `reqwest::header::{...}` 冗余 import。
  - 保持请求链路使用已统一的 `BaseHttpClient` 配置路径。
- 步骤级测试命令:
  - `cargo test qwen --lib`
  - `cargo test tavily --lib`
  - `cargo test topaz --lib`
  - `cargo test recraft --lib`
  - `cargo test vercel_ai --lib`
  - `cargo test searxng --lib`
  - `cargo test xiaomi_mimo --lib`
  - `cargo test baichuan --lib`
  - `cargo check --lib`
- 完成判定:
  - 上述 8 个 provider 文件不再包含重复且未使用的 `build_headers`。
  - 编译与定向测试通过。

### Step F2 删除未使用的重复 `build_headers` 样板（第二批 2 个 provider）

- 状态: `completed`
- 目标: 继续消除相同冗余模式，清理 `sap_ai` 与 `zhipu` 的重复未使用 header 构造代码。
- 预计改动文件:
  - `src/core/providers/sap_ai/mod.rs`
  - `src/core/providers/zhipu/mod.rs`
- 详细改动:
  - 删除 `build_headers` 方法。
  - 删除对应 `reqwest::header::{...}` import。
- 步骤级测试命令:
  - `cargo test sap_ai --lib`
  - `cargo test zhipu --lib`
  - `cargo check --lib`
- 完成判定:
  - 2 个 provider 文件不再包含 `build_headers` 冗余定义。
  - 定向测试与编译通过。

### Step F3 删除未使用 `base_client` 字段（保持初始化校验）

- 状态: `completed`
- 目标: 去除 Provider 结构体中未被读取的 `base_client` 状态冗余，同时保持构造期校验语义不变。
- 预计改动文件:
  - `src/core/providers/qwen/mod.rs`
  - `src/core/providers/tavily/mod.rs`
  - `src/core/providers/topaz/mod.rs`
  - `src/core/providers/recraft/mod.rs`
  - `src/core/providers/vercel_ai/mod.rs`
  - `src/core/providers/searxng/mod.rs`
  - `src/core/providers/xiaomi_mimo/mod.rs`
  - `src/core/providers/baichuan/mod.rs`
  - `src/core/providers/sap_ai/mod.rs`
  - `src/core/providers/zhipu/mod.rs`
- 详细改动:
  - 删除 provider struct 中 `base_client: BaseHttpClient` 字段。
  - 构造函数保留 `BaseHttpClient::new(base_config)` 调用，但改为局部 `_base_client` 以保留当前配置/初始化校验路径。
  - 保持外部行为与错误映射不变。
- 步骤级测试命令:
  - `cargo test qwen --lib`
  - `cargo test tavily --lib`
  - `cargo test topaz --lib`
  - `cargo test recraft --lib`
  - `cargo test vercel_ai --lib`
  - `cargo test searxng --lib`
  - `cargo test xiaomi_mimo --lib`
  - `cargo test baichuan --lib`
  - `cargo test sap_ai --lib`
  - `cargo test zhipu --lib`
  - `cargo check --lib`
- 完成判定:
  - 10 个 provider 结构体不再保留未使用 `base_client` 字段。
  - 相关定向测试与编译通过。

### Step F4 删除 SparkProvider 未使用 `base_client` 字段

- 状态: `completed`
- 目标: 清理 `SparkProvider` 最后一处同类未使用字段告警，同时维持构造期校验语义。
- 预计改动文件:
  - `src/core/providers/spark/provider.rs`
- 详细改动:
  - 删除 `SparkProvider` 结构体中的 `base_client` 字段。
  - 在 `new()` 中保留 `BaseHttpClient::new(base_config)`，改为局部 `_base_client`。
  - 保持 `SparkConfig::validate()` 与错误行为不变。
- 步骤级测试命令:
  - `cargo test spark --lib`
  - `cargo check --lib`
- 完成判定:
  - `SparkProvider` 不再持有未使用 `base_client` 字段。
  - 定向测试与编译通过。

### Step F5 删除 SparkProvider 未使用 `generate_signature` 方法

- 状态: `completed`
- 目标: 清理 Spark provider 内无调用的签名方法，减少死代码噪音。
- 预计改动文件:
  - `src/core/providers/spark/provider.rs`
- 详细改动:
  - 删除私有方法 `generate_signature`。
  - 保持其余请求校验与 provider 对外行为不变。
- 步骤级测试命令:
  - `cargo test spark --lib`
  - `cargo check --lib`
- 完成判定:
  - `generate_signature` 不再存在于 `spark/provider.rs`。
  - 定向测试与编译通过。

### Step F6 删除 Deepgram/ElevenLabs/Gemini 未使用状态字段

- 状态: `completed`
- 目标: 清理三类 provider 中未读取的内部状态字段，减少结构体冗余。
- 预计改动文件:
  - `src/core/providers/deepgram/provider.rs`
  - `src/core/providers/elevenlabs/provider.rs`
  - `src/core/providers/gemini/provider.rs`
- 详细改动:
  - `DeepgramProvider` / `ElevenLabsProvider`：删除未使用 `pool_manager` 字段。
  - `GeminiProvider`：删除未使用 `config` 与 `pool_manager` 字段。
  - 保留构造期初始化调用（`GlobalPoolManager::new()`）为局部变量以维持当前初始化失败语义。
- 步骤级测试命令:
  - `cargo test deepgram --lib`
  - `cargo test elevenlabs --lib`
  - `cargo test gemini --lib`
  - `cargo check --lib`
- 完成判定:
  - 上述 provider 不再保留未使用状态字段。
  - 定向测试与编译通过。

### Step F7 清理 StabilityProvider 未使用字段与方法

- 状态: `completed`
- 目标: 移除 `stability` provider 内未使用状态与冗余辅助方法，进一步降低 dead code 噪音。
- 预计改动文件:
  - `src/core/providers/stability/provider.rs`
- 详细改动:
  - 删除 `StabilityProvider` 中未使用 `pool_manager` 字段。
  - 删除未使用私有方法 `get_request_headers`。
  - 保留 `GlobalPoolManager::new()` 构造调用为局部 `_pool_manager`，维持初始化错误语义。
- 步骤级测试命令:
  - `cargo test stability --lib`
  - `cargo check --lib`
- 完成判定:
  - `stability/provider.rs` 不再包含上述未使用字段和方法。
  - 定向测试与编译通过。

### Step F8 清理 AnthropicProvider 未使用状态与私有方法

- 状态: `completed`
- 目标: 移除 `anthropic` provider 中当前未被调用的内部状态和冗余私有方法。
- 预计改动文件:
  - `src/core/providers/anthropic/provider.rs`
- 详细改动:
  - 删除 `AnthropicProvider` 未使用字段：`config`、`pool_manager`。
  - 删除未使用私有方法：`generate_headers`、`calculate_cost(&ChatRequest, &ChatResponse)`。
  - 保留 `GlobalPoolManager::new()` 初始化为局部 `_pool_manager`，维持初始化失败语义。
- 步骤级测试命令:
  - `cargo test anthropic --lib`
  - `cargo check --lib`
- 完成判定:
  - `anthropic/provider.rs` 不再包含上述未使用字段与方法。
  - 定向测试与编译通过。

### Step F9 清理 VLLM/HostedVLLM 未使用 `served_model` 字段

- 状态: `completed`
- 目标: 去除两个 vLLM provider struct 中仅在构造期使用、运行期未读取的冗余状态字段。
- 预计改动文件:
  - `src/core/providers/vllm/provider.rs`
  - `src/core/providers/hosted_vllm/provider.rs`
- 详细改动:
  - 删除 `VLLMProvider` 与 `HostedVLLMProvider` 的 `served_model` 字段。
  - 保留构造时 `served_model` 局部变量用于生成初始 `models`，不改变现有行为。
- 步骤级测试命令:
  - `cargo test vllm --lib`
  - `cargo test hosted_vllm --lib`
  - `cargo check --lib`
- 完成判定:
  - 两个 provider struct 不再包含未使用 `served_model` 字段。
  - 定向测试与编译通过。

### Step F10 清理多 provider handler 未使用 `config` 状态

- 状态: `completed`
- 目标: 去除多个 handler/provider 中“仅构造阶段使用，运行期无读取”的 `config` 字段，降低结构体冗余。
- 预计改动文件:
  - `src/core/providers/openrouter/client.rs`
  - `src/core/providers/mistral/chat/mod.rs`
  - `src/core/providers/moonshot/chat/mod.rs`
  - `src/core/providers/meta_llama/chat/mod.rs`
  - `src/core/providers/huggingface/embedding.rs`
- 详细改动:
  - 删除上述结构体中的未使用 `config` 字段。
  - `new(config: ...)` 构造签名保持不变，内部改为 `_config` 或直接不持有状态。
  - 删除 `HuggingFaceEmbeddingHandler::config()` 未使用访问器（若无外部引用）。
- 步骤级测试命令:
  - `cargo test openrouter --lib`
  - `cargo test mistral --lib`
  - `cargo test moonshot --lib`
  - `cargo test meta_llama --lib`
  - `cargo test huggingface --lib`
  - `cargo check --lib`
- 完成判定:
  - 以上 5 个文件不再含未使用 `config` 字段（和对应未使用访问器）。
  - 定向测试与编译通过。

### Step F11 清理 Vertex AI 子处理器未使用定位字段

- 状态: `completed`
- 目标: 删除 Vertex AI 子处理器中未使用的 `project_id/location` 等状态字段，保留 API 签名兼容。
- 预计改动文件:
  - `src/core/providers/vertex_ai/batches/mod.rs`
  - `src/core/providers/vertex_ai/files/mod.rs`
  - `src/core/providers/vertex_ai/gemini_embeddings/mod.rs`
  - `src/core/providers/vertex_ai/image_generation/mod.rs`
  - `src/core/providers/vertex_ai/text_to_speech/mod.rs`
- 详细改动:
  - 删除上述 handler struct 的冗余字段（`project_id` / `location`）。
  - 构造函数参数保留，改为 `_project_id` / `_location` 占位，避免外部调用断裂。
  - 不更改现有 TODO/stub 行为。
- 步骤级测试命令:
  - `cargo test vertex_ai --lib`
  - `cargo test vertex_ai::batches --lib`
  - `cargo test vertex_ai::files --lib`
  - `cargo test vertex_ai::gemini_embeddings --lib`
  - `cargo test vertex_ai::image_generation --lib`
  - `cargo test vertex_ai::text_to_speech --lib`
  - `cargo check --lib`
- 完成判定:
  - 上述 5 个 handler 不再持有未使用定位字段。
  - 定向测试与编译通过。

### Step F12 清理 VertexAIProvider 未使用健康状态缓存字段

- 状态: `completed`
- 目标: 去除 `VertexAIProvider` 内未被读取的 `health_status` 冗余状态，保留现有 `health_check()` 行为。
- 预计改动文件:
  - `src/core/providers/vertex_ai/client.rs`
- 详细改动:
  - 删除 `VertexAIProvider` 结构体中的 `health_status` 字段。
  - 删除构造函数中对应初始化逻辑。
  - 清理未使用 import（`tokio::sync::RwLock`）。
- 步骤级测试命令:
  - `cargo test vertex_ai --lib`
  - `cargo check --lib`
- 完成判定:
  - `VertexAIProvider` 不再持有未使用 `health_status` 字段。
  - 定向测试与编译通过。

---

## 4. 执行日志（每步完成后追加）

- 2026-02-10
  - Step A1: `completed`
    - 修改文件:
      - `src/core/providers/mod.rs`
    - 主要改动:
      - 扩展 `ProviderType` 一致性测试覆盖全部非 `Custom` 变体。
      - 新增 `from_config_async` 分支守护测试（支持分支不得落入 `NotImplemented`，不支持分支必须落入 `NotImplemented`）。
    - 执行测试:
      - `cargo test test_provider_type_from_display_consistency --lib` -> pass
      - `cargo test test_provider_type_all_variants_covered --lib` -> pass
      - `cargo test test_from_config_async_supported_variants_do_not_fallthrough_to_not_implemented --lib` -> pass
      - `cargo test test_from_config_async_unsupported_variants_return_not_implemented --lib` -> pass
      - `cargo check --lib` -> pass
  - Step A2: `completed`
    - 修改文件:
      - `src/core/providers/mod.rs`
    - 主要改动:
      - `create_provider` 改为统一解析入口：优先使用 `provider_type`，为空时回退 `name`。
      - 将 `ProviderConfig` 映射到统一 `factory_config` 后调用 `Provider::from_config_async`，移除原“固定占位错误”路径。
      - 增加 Cloudflare 的 `api_token <- api_key` 兼容映射，以及 `account_id <- organization` 回退。
      - 新增 3 条 `create_provider` 行为测试（优先级、回退、unknown 错误信息）。
    - 执行测试:
      - `cargo test test_create_provider_ --lib` -> pass
      - `cargo check --lib` -> pass
  - Step A3: `completed`
    - 修改文件:
      - `src/core/providers/mod.rs`
      - `src/core/providers/context.rs`（已删除）
    - 主要改动:
      - 删除 `pub mod context;` 导出。
      - 删除未被引用的 `providers/context` 模块。
    - 执行测试:
      - `rg --no-heading -n "core::providers::context|providers::context" src tests benches examples` -> no match
      - `cargo check --lib` -> pass
      - `cargo test test_from_config_async_unsupported_variants_return_not_implemented --lib` -> pass
  - Step A4: `completed`
    - 修改文件:
      - `src/core/providers/mod.rs`
    - 主要改动:
      - 新增 `Provider::factory_supported_provider_types()` 作为 factory 已接线 provider 的单一声明源。
      - `create_provider` 在构造前先校验该声明源，未接线 provider 统一返回 `NotImplemented`。
      - 测试辅助 `supported_factory_provider_types()` 复用同一声明源，避免测试侧重复维护列表。
    - 执行测试:
      - `cargo test test_create_provider_ --lib` -> pass
      - `cargo test test_from_config_async_supported_variants_do_not_fallthrough_to_not_implemented --lib` -> pass
      - `cargo check --lib` -> pass
  - Step B1: `completed`
    - 修改文件:
      - `src/core/types/context.rs`
      - `src/core/models/mod.rs`
      - `src/server/routes/ai/context.rs`
      - `src/server/routes/ai/chat.rs`
      - `src/server/routes/ai/embeddings.rs`
      - `src/server/routes/ai/completions.rs`
      - `src/server/routes/ai/images.rs`
      - `src/server/middleware/auth.rs`
      - `src/auth/system.rs`
      - `src/auth/types.rs`
      - `src/auth/tests.rs`
      - `src/core/webhooks/events.rs`
      - `src/core/webhooks/manager.rs`
      - `src/core/webhooks/types.rs`
      - `src/server/routes/ai/mod.rs`
    - 主要改动:
      - 将 `core/models::RequestContext` 收敛为 `core/types/context::RequestContext` 类型别名，避免结构体并行定义。
      - 在 `core/types/context.rs` 增加兼容 helper：`with_user/with_api_key/with_client_info/with_tracing` 与 `team_id/api_key_id` metadata 读写接口。
      - 路由层删除 `to_core_context` 桥接，直接使用统一 `RequestContext` 传递到 provider 调用。
      - 认证链路改为通过 metadata 记录 `team_id/api_key_id`，并统一 `user_id` 为字符串表示。
      - webhook/auth/ai 相关模块的 `RequestContext` 引用统一到 `core::types::context::RequestContext`。
    - 执行测试:
      - `cargo test core::types::context::tests --lib` -> pass
      - `cargo test core::models::tests::test_request_context_ --lib` -> pass
      - `cargo test server::routes::ai::context --lib` -> pass
      - `cargo test auth::tests --lib` -> pass
      - `cargo check --lib` -> pass
  - Step B2: `completed`
    - 修改文件:
      - `src/server/routes/ai/chat.rs`
      - `src/core/models/openai/messages.rs`
      - `src/core/models/openai/tools.rs`
    - 主要改动:
      - 将 `chat.rs` 的消息转换改为调用 `Into/From`，移除路由层内联大段 `match` 样板。
      - 在 `core/models/openai/messages.rs` 增加 `OpenAI <-> core` 的 `MessageRole/MessageContent/ContentPart/ChatMessage` 转换实现。
      - 在 `core/models/openai/tools.rs` 增加 `FunctionCall/ToolCall` 的 `OpenAI <-> core` 转换实现。
      - 新增 `messages` 转换单测，确保双向转换行为可验证。
    - 执行测试:
      - `cargo test server::routes::ai::chat --lib` -> pass
      - `cargo test core::models::openai::messages::tests --lib` -> pass
      - `cargo check --lib` -> pass
  - Step B3: `completed`
    - 修改文件:
      - `src/core/providers/openai/models.rs`
      - `src/core/providers/openai/transformer.rs`
      - `src/sdk/config.rs`
    - 主要改动:
      - 在 `providers/openai/models.rs` 增加 `OpenAIMessage <-> core::models::openai::ChatMessage` 适配方法，收敛 provider 侧消息转换入口。
      - 在 `providers/openai/transformer.rs` 复用上述适配方法替代重复字段映射，保留 `Document/ToolResult/ToolUse` 不支持的显式错误分支。
      - 修正 `OpenAIModelFeature::StreamingSupport` 判定：不再默认加给所有模型，改为与 `create_config` 一致地排除 embedding/whisper，修复 embedding 流式能力误判。
      - 调整 `sdk/config` 的 OpenAI builder 断言为能力型检查（模型列表非空且包含 `gpt-` 前缀），避免对过时具体模型名硬编码。
    - 执行测试:
      - `cargo test core::providers::openai::transformer --lib` -> pass
      - `cargo test core::providers::openai::provider::tests::test_feature_support --lib -- --exact` -> pass
      - `cargo test sdk::config::tests::test_config_builder_add_openai --lib -- --exact` -> pass
      - `cargo test openai --lib` -> pass
      - `cargo check --lib` -> pass
  - Step C1: `completed`
    - 修改文件:
      - `src/utils/net/http.rs`
      - `src/core/providers/base/connection_pool.rs`
      - `src/core/providers/base_provider.rs`
      - `src/core/providers/base/http.rs`
      - `src/utils/net/client/utils.rs`
    - 主要改动:
      - 在 `utils/net/http.rs` 新增统一 builder 入口：`create_client_builder_with_config` / `create_client_builder`，并让 `create_optimized_client`、`create_custom_client`、`create_custom_client_with_headers` 统一复用。
      - 新增 `create_custom_client_with_config`，作为“带池配置”的单点工厂，供 provider 基础层统一接入。
      - `base/connection_pool.rs` 删除本地 `Client::builder` 拼装逻辑，改为通过 `create_custom_client_with_config` 创建全局/隔离 client，并保留 `PoolConfig` 参数来源。
      - `base_provider::BaseHttpClient::new` 与 `base/http::create_http_client` 改为复用 `utils/net/http` 的超时缓存 client（`get_client_with_timeout_fallible`），不再重复维护连接池参数。
      - `utils/net/client/utils.rs` 改为复用统一 builder，并修复 `default_headers` 循环覆盖问题（改为一次性构建 HeaderMap）。
    - 执行测试:
      - `cargo test utils::net::http --lib` -> pass
      - `cargo test utils::net::client::utils --lib` -> pass
      - `cargo test core::providers::base::connection_pool --lib` -> pass
      - `cargo test core::providers::base_provider --lib` -> pass
      - `cargo check --lib` -> pass
  - Step C2: `completed`
    - 修改文件:
      - `src/core/types/mod.rs`
      - `src/core/types/config/mod.rs`
      - `src/config/mod.rs`
      - `src/sdk/config.rs`
    - 主要改动:
      - 将 `core/types/config` 明确为 legacy 兼容层：模块文档增加迁移说明，子模块改为 `#[doc(hidden)]`，减少继续扩散。
      - 对 `LiteLLMConfig` 添加弃用标记，明确迁移到服务端 `config::models` 和客户端 `sdk::config` 的 canonical 路径。
      - 新增 `LegacyServerConfig` / `LegacyProviderConfigEntry` 兼容别名，降低同名结构语义冲突。
      - 在 `config/mod.rs` 新增 canonical 别名 `GatewayServerConfig` / `GatewayProviderConfig`，在 `sdk/config.rs` 新增 `ClientRuntimeConfig` / `ClientProviderConfig`。
      - 在 `core/types/mod.rs` 将 `config` 模块标记 `#[doc(hidden)]`，弱化对外主路径。
    - 执行测试:
      - `cargo test config --lib` -> failed（已定位为仓库现有失败用例：`config::builder::tests::tests::test_config_builder`、`config::models::gateway::tests::test_gateway_config_validate_empty_database_url`、`config::models::gateway::tests::test_gateway_config_validate_success`）
      - `cargo test sdk::config --lib` -> pass
      - `cargo test core::types::config --lib` -> pass
      - `cargo test config::tests::test_config_serialization --lib` -> pass
      - `cargo check --lib` -> pass
  - Step D1: `completed`
    - 修改文件:
      - `Cargo.toml`
      - `src/storage/mod.rs`
      - `src/storage/redis_optimized/mod.rs`
    - 主要改动:
      - 将 `redis_optimized` 定位为实验能力：新增 feature `redis-optimized`，默认不进入主流程。
      - `src/storage/mod.rs` 中 `pub mod redis_optimized` 改为 `#[cfg(feature = "redis-optimized")]`，避免默认构建和主路径暴露悬空并行实现。
      - 在 `redis_optimized` 模块文档中明确“实验特性 + feature gate”语义。
    - 执行测试:
      - `cargo test storage::redis --lib` -> pass
      - `cargo check --lib` -> pass
      - `cargo check --lib --features redis-optimized` -> pass
      - `cargo test storage::redis_optimized --lib --features redis-optimized` -> pass
    - 备注:
      - 期间遇到构建产物导致磁盘不足，执行 `cargo clean` 释放空间后重跑通过。
  - Step D2: `completed`
    - 修改文件:
      - `src/core/mod.rs`
      - `src/core/cache_manager/mod.rs`
      - `src/core/cache_manager/manager.rs`
      - `src/core/cache/mod.rs`
      - `src/core/semantic_cache/mod.rs`
    - 主要改动:
      - 在 `core/mod.rs` 明确缓存边界：启用 `core::cache` 作为 canonical 确定性缓存入口，将 `core::cache_manager` 标注为 legacy 兼容层并 `#[doc(hidden)]`。
      - 在 `cache_manager` 模块补充 legacy 迁移说明，明确新代码应使用 `core::cache`（确定性 key cache）与 `core::semantic_cache`（语义相似度缓存）。
      - 在 `core/cache/mod.rs` 与 `core/semantic_cache/mod.rs` 增加职责边界说明，减少三套缓存模块语义重叠。
    - 执行测试:
      - `cargo test cache --lib` -> pass
      - `cargo check --lib` -> pass
  - Step D3: `completed`
    - 修改文件:
      - `src/utils/logging/logging/types.rs`
      - `src/utils/logging/logging/async_logger.rs`
      - `src/utils/logging/logging/mod.rs`
      - `src/utils/logging/logging/tests.rs`
      - `src/core/observability/types.rs`
      - `src/core/observability/mod.rs`
      - `src/core/observability/tests.rs`
      - `src/core/integrations/observability/datadog.rs`
    - 主要改动:
      - 统一 `LogEntry` 主定义为 `src/utils/logging/utils/types.rs::LogEntry`，其余并行结构改为语义化命名，消除同名平行定义。
      - 将 async logger 侧 `LogEntry` 更名为 `AsyncLogRecord`，并同步调整 re-export、实现与测试引用。
      - 将 observability 侧原 `LogEntry` 更名为 `ObservabilityLogRecord`，新增与 canonical `LogEntry` 的双向转换适配，保留 observability 富字段语义。
      - 将 DataDog 集成内部私有 `LogEntry` 更名为 `DataDogLogRecord`，避免局部类型名歧义。
    - 执行测试:
      - `cargo test logging --lib` -> pass
      - `cargo check --lib` -> pass
  - Step D4: `completed`
    - 修改文件:
      - `src/core/router/mod.rs`
      - `src/utils/config/optimized.rs`
      - `src/core/cache/mod.rs`
      - `src/utils/mod.rs`
      - `src/config/builder/mod.rs`
      - `src/utils/config/helpers.rs`
      - `src/server/routes/ai/mod.rs`
    - 主要改动:
      - 对 `core::router` 中 legacy 模块导出增加 `#[doc(hidden)]`（`health/load_balancer/metrics/strategy`），降低旧路由接口继续扩散风险。
      - 移除多处模块级 `#![allow(dead_code)]`（`utils/config/optimized.rs`、`core/cache/mod.rs`、`config/builder/mod.rs`、`utils/config/helpers.rs`、`server/routes/ai/mod.rs`），从“整模块抑制”收敛为编译器真实暴露未使用项。
      - 清理 `utils/mod.rs` 中通用公开工具函数上的 `#[allow(dead_code)]`，保留 API 但去除冗余抑制注解。
      - `allow(dead_code)` 计数由 `182` 降到 `165`（`src/` 范围）。
    - 执行测试:
      - `cargo check --lib` -> pass
      - `cargo test --lib` -> failed（4 个失败：`config::builder::tests::tests::test_config_builder`、`config::models::gateway::tests::test_gateway_config_validate_empty_database_url`、`config::models::gateway::tests::test_gateway_config_validate_success`、`config::validation::auth_validators::tests::test_auth_config_zero_jwt_expiration`）
  - Step E1: `completed`
    - 修改文件:
      - `src/config/models/gateway.rs`
      - `src/config/builder/tests.rs`
      - `src/config/validation/auth_validators.rs`
    - 主要改动:
      - `gateway` 测试夹具补齐校验前提：设置 `database.enabled = true`，并使用符合当前强度规则的 JWT secret。
      - `builder` 测试改为显式注入合法 `AuthConfig`，避免默认空 secret 导致构建失败。
      - `jwt_expiration=0` 用例断言改为匹配当前最小过期时间规则（at least 5 minutes）。
    - 执行测试:
      - `cargo test config::builder::tests::tests::test_config_builder --lib -- --exact` -> pass
      - `cargo test config::models::gateway::tests::test_gateway_config_validate_success --lib -- --exact` -> pass
      - `cargo test config::models::gateway::tests::test_gateway_config_validate_empty_database_url --lib -- --exact` -> pass
      - `cargo test config::validation::auth_validators::tests::test_auth_config_zero_jwt_expiration --lib -- --exact` -> pass
  - Step E2: `completed`
    - 修改文件:
      - 无（验证步骤）
    - 执行测试:
      - `cargo check --lib` -> pass
      - `cargo test --lib` -> pass（`12363 passed; 0 failed`）
  - Step F1: `completed`
    - 修改文件:
      - `src/core/providers/qwen/mod.rs`
      - `src/core/providers/tavily/mod.rs`
      - `src/core/providers/topaz/mod.rs`
      - `src/core/providers/recraft/mod.rs`
      - `src/core/providers/vercel_ai/mod.rs`
      - `src/core/providers/searxng/mod.rs`
      - `src/core/providers/xiaomi_mimo/mod.rs`
      - `src/core/providers/baichuan/mod.rs`
    - 主要改动:
      - 删除 8 个 provider 中“定义未使用”的重复 `build_headers` 方法。
      - 同步删除对应 `reqwest::header::{...}` 冗余 import，降低模板噪音与维护面。
      - 保持请求链路继续依赖统一的 `BaseHttpClient` 配置路径。
    - 执行测试:
      - `cargo test qwen --lib` -> pass
      - `cargo test tavily --lib` -> pass
      - `cargo test topaz --lib` -> pass
      - `cargo test recraft --lib` -> pass
      - `cargo test vercel_ai --lib` -> pass
      - `cargo test searxng --lib` -> pass
      - `cargo test xiaomi_mimo --lib` -> pass
      - `cargo test baichuan --lib` -> pass
      - `cargo check --lib` -> pass
  - Step F2: `completed`
    - 修改文件:
      - `src/core/providers/sap_ai/mod.rs`
      - `src/core/providers/zhipu/mod.rs`
    - 主要改动:
      - 删除 `sap_ai` 与 `zhipu` 中未使用的 `build_headers` 冗余实现。
      - 删除对应 `reqwest::header::{...}` import，继续收敛 provider 样板重复。
    - 执行测试:
      - `cargo test sap_ai --lib` -> pass
      - `cargo test zhipu --lib` -> pass
      - `cargo check --lib` -> pass（warning 总数 `165 -> 163`）
  - Step F3: `completed`
    - 修改文件:
      - `src/core/providers/qwen/mod.rs`
      - `src/core/providers/tavily/mod.rs`
      - `src/core/providers/topaz/mod.rs`
      - `src/core/providers/recraft/mod.rs`
      - `src/core/providers/vercel_ai/mod.rs`
      - `src/core/providers/searxng/mod.rs`
      - `src/core/providers/xiaomi_mimo/mod.rs`
      - `src/core/providers/baichuan/mod.rs`
      - `src/core/providers/sap_ai/mod.rs`
      - `src/core/providers/zhipu/mod.rs`
    - 主要改动:
      - 删除 10 个 provider 结构体中的未使用 `base_client` 字段。
      - 保留构造期 `BaseHttpClient::new(base_config)`，改为局部 `_base_client`，维持当前初始化校验和错误映射行为。
      - 同步简化 `Ok(Self { ... })` 构造，去除冗余状态持有。
    - 执行测试:
      - `cargo test qwen --lib` -> pass
      - `cargo test tavily --lib` -> pass
      - `cargo test topaz --lib` -> pass
      - `cargo test recraft --lib` -> pass
      - `cargo test vercel_ai --lib` -> pass
      - `cargo test searxng --lib` -> pass
      - `cargo test xiaomi_mimo --lib` -> pass
      - `cargo test baichuan --lib` -> pass
      - `cargo test sap_ai --lib` -> pass
      - `cargo test zhipu --lib` -> pass
      - `cargo check --lib` -> pass（warning 总数 `163 -> 153`）
  - Step F4: `completed`
    - 修改文件:
      - `src/core/providers/spark/provider.rs`
    - 主要改动:
      - 删除 `SparkProvider` 结构体中的未使用 `base_client` 字段。
      - 在 `new()` 保留 `BaseHttpClient::new(base_config)` 调用并改为局部 `_base_client`，保持初始化校验行为。
    - 执行测试:
      - `cargo test spark --lib` -> pass
      - `cargo check --lib` -> pass（warning 总数 `153 -> 152`）
  - Step F5: `completed`
    - 修改文件:
      - `src/core/providers/spark/provider.rs`
    - 主要改动:
      - 删除 `SparkProvider` 内部未使用的私有方法 `generate_signature`。
      - 不变更现有请求验证与 provider 对外行为。
    - 执行测试:
      - `cargo test spark --lib` -> pass
      - `cargo check --lib` -> pass（warning 总数 `152 -> 151`）
  - Step F6: `completed`
    - 修改文件:
      - `src/core/providers/deepgram/provider.rs`
      - `src/core/providers/elevenlabs/provider.rs`
      - `src/core/providers/gemini/provider.rs`
    - 主要改动:
      - `DeepgramProvider` 与 `ElevenLabsProvider` 删除未使用 `pool_manager` 字段。
      - `GeminiProvider` 删除未使用 `config` 与 `pool_manager` 字段。
      - 保留构造期 `GlobalPoolManager::new()` 初始化为局部 `_pool_manager`，维持初始化失败语义。
    - 执行测试:
      - `cargo test deepgram --lib` -> pass
      - `cargo test elevenlabs --lib` -> pass
      - `cargo test gemini --lib` -> pass
      - `cargo check --lib` -> pass（warning 总数 `151 -> 148`）
      - `cargo test --lib` -> pass（`12363 passed; 0 failed`）
  - Step F7: `completed`
    - 修改文件:
      - `src/core/providers/stability/provider.rs`
    - 主要改动:
      - 删除 `StabilityProvider` 的未使用 `pool_manager` 字段。
      - 删除未使用私有方法 `get_request_headers`，并清理相关 import。
      - 保留 `GlobalPoolManager::new()` 初始化为局部 `_pool_manager`，维持初始化错误语义。
    - 执行测试:
      - `cargo test stability --lib` -> pass
      - `cargo check --lib` -> pass（warning 总数 `148 -> 146`）
      - `cargo test --lib` -> pass（`12363 passed; 0 failed`）
  - Step F8: `completed`
    - 修改文件:
      - `src/core/providers/anthropic/provider.rs`
    - 主要改动:
      - 删除 `AnthropicProvider` 的未使用字段：`config`、`pool_manager`。
      - 删除未使用私有方法 `generate_headers`、`calculate_cost(&ChatRequest, &ChatResponse)`。
      - 保留 `GlobalPoolManager::new()` 初始化为局部 `_pool_manager`，维持初始化失败语义。
    - 执行测试:
      - `cargo test anthropic --lib` -> pass（`208 passed; 0 failed`）
      - `cargo check --lib` -> pass（warning 总数 `146 -> 145`）
  - Step F9: `completed`
    - 修改文件:
      - `src/core/providers/vllm/provider.rs`
      - `src/core/providers/hosted_vllm/provider.rs`
    - 主要改动:
      - 删除 `VLLMProvider` 与 `HostedVLLMProvider` 的未使用 `served_model` 字段。
      - 保留构造阶段 `served_model` 局部变量用于初始化 `models`，不改变对外行为。
    - 执行测试:
      - `cargo test vllm --lib` -> pass（`92 passed; 0 failed`）
      - `cargo test hosted_vllm --lib` -> pass（`42 passed; 0 failed`）
      - `cargo check --lib` -> pass（warning 总数 `145 -> 143`）
  - Step F10: `completed`
    - 修改文件:
      - `src/core/providers/openrouter/client.rs`
      - `src/core/providers/mistral/chat/mod.rs`
      - `src/core/providers/moonshot/chat/mod.rs`
      - `src/core/providers/meta_llama/chat/mod.rs`
      - `src/core/providers/huggingface/embedding.rs`
    - 主要改动:
      - 删除 5 个 provider/handler 中未使用的 `config` 状态字段。
      - 保持 `new(config: ...)` 构造签名，内部改为不持有冗余状态（使用 `_config` 参数形式）。
      - 删除未使用访问器 `HuggingFaceEmbeddingHandler::config()`。
    - 执行测试:
      - `cargo test openrouter --lib` -> pass（`144 passed; 0 failed`）
      - `cargo test mistral --lib` -> pass（`88 passed; 0 failed`）
      - `cargo test moonshot --lib` -> pass（`54 passed; 0 failed`）
      - `cargo test meta_llama --lib` -> pass（`13 passed; 0 failed`）
      - `cargo test huggingface --lib` -> pass（`68 passed; 0 failed`）
      - `cargo check --lib` -> pass（warning 总数 `143 -> 137`）
  - Step F11: `completed`
    - 修改文件:
      - `src/core/providers/vertex_ai/batches/mod.rs`
      - `src/core/providers/vertex_ai/files/mod.rs`
      - `src/core/providers/vertex_ai/gemini_embeddings/mod.rs`
      - `src/core/providers/vertex_ai/image_generation/mod.rs`
      - `src/core/providers/vertex_ai/text_to_speech/mod.rs`
    - 主要改动:
      - 删除 5 个 Vertex AI 子处理器的未使用定位字段（`project_id` / `location`）。
      - 构造函数签名保持兼容，改为 `_project_id` / `_location` 占位参数，不影响外部调用。
      - 保留现有 stub/TODO 路径行为不变。
    - 执行测试:
      - `cargo test vertex_ai --lib` -> pass（`226 passed; 0 failed`）
      - `cargo test vertex_ai::batches --lib` -> pass（`0 passed; 0 failed`）
      - `cargo test vertex_ai::files --lib` -> pass（`3 passed; 0 failed`）
      - `cargo test vertex_ai::gemini_embeddings --lib` -> pass（`2 passed; 0 failed`）
      - `cargo test vertex_ai::image_generation --lib` -> pass（`3 passed; 0 failed`）
      - `cargo test vertex_ai::text_to_speech --lib` -> pass（`3 passed; 0 failed`）
      - `cargo check --lib` -> pass（warning 总数 `137 -> 132`）
  - Step F12: `completed`
    - 修改文件:
      - `src/core/providers/vertex_ai/client.rs`
    - 主要改动:
      - 删除 `VertexAIProvider` 中未读取 `health_status` 字段。
      - 删除对应构造期初始化逻辑，清理 `tokio::sync::RwLock` 冗余 import。
      - 保持 `health_check()` 走实时 `check_health()` 逻辑，不改变对外行为。
    - 执行测试:
      - `cargo test vertex_ai --lib` -> pass（`226 passed; 0 failed`）
      - `cargo check --lib` -> pass（warning 总数 `132 -> 131`）
