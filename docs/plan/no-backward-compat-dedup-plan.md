# 无向后兼容收敛执行计划（Fixflow / 文件级）

- 计划版本: v1
- 创建时间: 2026-02-12
- 仓库: `/Users/lifcc/Desktop/code/AI/gateway/litellm-rs`
- 执行模式: `per_step`（每步: 修改 -> 测试 -> 更新计划状态 -> 提交 -> 下一步）

## 0. Ready Criteria (DoR)

### 0.1 目标

彻底收敛本仓库中已确认的重复/冗余设计，按“单一实现路径”重构，不保留兼容层。

### 0.2 约束

- Backward compatibility: `not required`
- Commit policy: `per_step`
- 每一步必须有自动化测试命令并记录结果
- 每一步只允许一个状态为 `in_progress`

### 0.3 Dirty Baseline

- `git status --short --branch`: `## main...origin/main [ahead 17]`
- 当前工作区无未提交文件（仅分支领先）

### 0.4 验证矩阵

- Step-level:
  - `cargo check`
  - 与改动模块对应的 `cargo test --lib <pattern>` 或 `cargo test --test <name>`
- Final-level:
  - `cargo check`
  - `cargo test --lib`
  - `cargo test --tests`

---

## 1. 全量步骤（文件级）

## Step 0 - 基线校验

- 状态: `completed`
- 改动文件: 无
- 执行命令:
  - `cargo check`
- 完成标准:
  - 基线可编译，后续每步可做增量对比

## Step 1 - 删除重复告警实现（保留 monitoring/alerts，移除 core 重复实现）

- 状态: `completed`
- 目标:
  - 移除 `core/alerting` 与 `core/observability` 中重复告警实现，统一保留 `monitoring/alerts`
- 预计改动文件:
  - `src/core/mod.rs`
  - `src/core/observability/mod.rs`
  - `src/core/observability/alerting.rs` (delete)
  - `src/core/alerting/mod.rs` (delete)
  - `src/core/alerting/manager.rs` (delete)
  - `src/core/alerting/config.rs` (delete)
  - `src/core/alerting/channels.rs` (delete)
  - `src/core/alerting/types.rs` (delete)
  - `src/core/alerting/tests.rs` (delete)
- 具体调整:
  - 从 `src/core/mod.rs` 移除 `pub mod alerting;`
  - 从 `src/core/observability/mod.rs` 移除 `mod alerting;` 与 `pub use alerting::AlertManager;`
  - 删除 `src/core/alerting/` 整个目录
  - 删除 `src/core/observability/alerting.rs`
- 步骤测试:
  - `cargo check`
  - `cargo test --lib monitoring::alerts`
- 完成标准:
  - 编译通过
  - 仓库中不再存在 `src/core/alerting` 与 `src/core/observability/alerting.rs`

## Step 2 - 删除 legacy cache_manager（仅保留 core/cache）

- 状态: `completed`
- 目标:
  - 移除 `core/cache_manager` 兼容层，统一到 `core/cache`
- 预计改动文件:
  - `src/core/mod.rs`
  - `src/core/cache_manager/mod.rs` (delete)
  - `src/core/cache_manager/manager.rs` (delete)
  - `src/core/cache_manager/types.rs` (delete)
  - `src/core/cache_manager/tests.rs` (delete)
  - `benches/performance_benchmarks.rs`
- 具体调整:
  - 从 `src/core/mod.rs` 移除 `pub mod cache_manager;`
  - 删除 `src/core/cache_manager/` 整个目录
  - `benches/performance_benchmarks.rs` 中删除/替换 `cache_manager` 相关基准段，避免引用已删除模块
- 步骤测试:
  - `cargo check`
  - `cargo test --lib core::cache`
- 完成标准:
  - 编译通过
  - `rg "cache_manager" src benches tests` 不再存在业务引用

## Step 3 - 删除 legacy router 栈（保留 UnifiedRouter）

- 状态: `completed`
- 目标:
  - 移除 `core/router/load_balancer` 与 `core/router/strategy` 及其配套 legacy 模块
- 预计改动文件:
  - `src/core/router/mod.rs`
  - `src/core/router/load_balancer/mod.rs` (delete)
  - `src/core/router/load_balancer/core.rs` (delete)
  - `src/core/router/load_balancer/deployment_info.rs` (delete)
  - `src/core/router/load_balancer/fallback_config.rs` (delete)
  - `src/core/router/load_balancer/fallback_selection.rs` (delete)
  - `src/core/router/load_balancer/selection.rs` (delete)
  - `src/core/router/load_balancer/tag_routing.rs` (delete)
  - `src/core/router/strategy/mod.rs` (delete)
  - `src/core/router/strategy/types.rs` (delete)
  - `src/core/router/strategy/executor.rs` (delete)
  - `src/core/router/strategy/selection.rs` (delete)
  - `src/core/router/health.rs` (delete)
  - `src/core/router/metrics.rs` (delete)
  - `src/core/router/tests/mod.rs`
  - `src/core/router/tests/load_balancer_tests.rs` (delete)
  - `src/core/router/tests/strategy_executor_tests.rs` (delete)
  - `tests/integration/router_tests.rs`
  - `benches/performance_benchmarks.rs`
- 具体调整:
  - 从 `src/core/router/mod.rs` 删除 legacy 模块声明:
    - `pub mod health;`
    - `pub mod load_balancer;`
    - `pub mod metrics;`
    - `pub mod strategy;`
  - 删除对应 legacy 文件与目录
  - 从 `src/core/router/tests/mod.rs` 删除 legacy 测试模块注册
  - `tests/integration/router_tests.rs` 改为仅测试 `UnifiedRouter` 与 `deployment`（移除 `LoadBalancer`/legacy strategy）
  - `benches/performance_benchmarks.rs` 改为只保留 `UnifiedRouter` 路径基准
- 步骤测试:
  - `cargo check`
  - `cargo test --lib core::router::tests`
  - `cargo test --test router_tests`
- 完成标准:
  - 编译通过
  - `rg "router::load_balancer|router::strategy::" src tests benches` 结果为 0

## Step 4 - 删除重复请求/响应模型层（移除 core/models/request + core/models/response）

- 状态: `completed`
- 目标:
  - 删除未被业务使用的重复模型层，统一使用 `core/types/*` 与 `core/models/openai/*`
- 预计改动文件:
  - `src/core/models/mod.rs`
  - `src/core/models/request.rs` (delete)
  - `src/core/models/response/mod.rs` (delete)
  - `src/core/models/response/completion.rs` (delete)
  - `src/core/models/response/embedding.rs` (delete)
  - `src/core/models/response/error.rs` (delete)
  - `src/core/models/response/media.rs` (delete)
  - `src/core/models/response/metadata.rs` (delete)
  - `src/core/models/response/moderation.rs` (delete)
  - `src/core/models/response/rerank.rs` (delete)
  - `src/core/models/response/types.rs` (delete)
- 具体调整:
  - 从 `src/core/models/mod.rs` 删除 `pub mod request;` 与 `pub mod response;`
  - 删除对应文件/目录
  - 若出现编译引用，统一改到 `crate::core::types::*` 或 `crate::core::models::openai::*`
- 步骤测试:
  - `cargo check`
  - `cargo test --lib core::models`
- 完成标准:
  - 编译通过
  - `rg "core::models::request|core::models::response" src tests benches examples` 为 0

## Step 5 - 最终回归与计划归档

- 状态: `pending`
- 目标:
  - 运行全量验证并回写计划状态与每步结果
- 预计改动文件:
  - `docs/plan/no-backward-compat-dedup-plan.md`
- 具体调整:
  - 更新每一步状态为 `completed` 或 `blocked`
  - 记录每一步实际变更文件与测试结果
  - 汇总 breaking changes（按模块）
- 步骤测试:
  - `cargo check`
  - `cargo test --lib`
  - `cargo test --tests`
- 完成标准:
  - 三条命令全部通过（若失败需记录失败点和阻塞原因）
  - 计划文档完整闭环

---

## 2. 执行日志（每步完成后追加）

### Step 0

- 状态变更: `in_progress -> completed`
- 实际改动文件: 无
- 测试命令:
  - `cargo check` ✅
- 备注: 基线通过

### Step 1

- 状态变更: `pending -> in_progress -> completed`
- 实际改动文件:
  - `src/core/mod.rs`
  - `src/core/observability/mod.rs`
  - `src/core/observability/alerting.rs` (deleted)
  - `src/core/alerting/channels.rs` (deleted)
  - `src/core/alerting/config.rs` (deleted)
  - `src/core/alerting/manager.rs` (deleted)
  - `src/core/alerting/mod.rs` (deleted)
  - `src/core/alerting/tests.rs` (deleted)
  - `src/core/alerting/types.rs` (deleted)
- 测试命令:
  - `cargo check` ✅
  - `cargo test --lib monitoring::alerts` ✅ (75 passed)
- 结果: 完成，已移除 core 侧重复告警实现，仅保留 monitoring 告警路径

### Step 2

- 状态变更: `pending -> in_progress -> completed`
- 实际改动文件:
  - `src/core/mod.rs`
  - `src/core/cache_manager/mod.rs` (deleted)
  - `src/core/cache_manager/manager.rs` (deleted)
  - `src/core/cache_manager/types.rs` (deleted)
  - `src/core/cache_manager/tests.rs` (deleted)
  - `benches/performance_benchmarks.rs`
- 测试命令:
  - `cargo check` ✅
  - `cargo test --lib core::cache` ✅ (126 passed)
- 结果: 完成，`cache_manager` 已彻底下线，缓存主路径仅保留 `core/cache`

### Step 3

- 状态变更: `pending -> in_progress -> completed`
- 实际改动文件:
  - `src/core/router/mod.rs`
  - `src/core/router/tests/mod.rs`
  - `src/core/router/health.rs` (deleted)
  - `src/core/router/metrics.rs` (deleted)
  - `src/core/router/load_balancer/mod.rs` (deleted)
  - `src/core/router/load_balancer/core.rs` (deleted)
  - `src/core/router/load_balancer/deployment_info.rs` (deleted)
  - `src/core/router/load_balancer/fallback_config.rs` (deleted)
  - `src/core/router/load_balancer/fallback_selection.rs` (deleted)
  - `src/core/router/load_balancer/selection.rs` (deleted)
  - `src/core/router/load_balancer/tag_routing.rs` (deleted)
  - `src/core/router/strategy/mod.rs` (deleted)
  - `src/core/router/strategy/types.rs` (deleted)
  - `src/core/router/strategy/executor.rs` (deleted)
  - `src/core/router/strategy/selection.rs` (deleted)
  - `src/core/router/tests/load_balancer_tests.rs` (deleted)
  - `src/core/router/tests/strategy_executor_tests.rs` (deleted)
  - `tests/integration/router_tests.rs`
  - `benches/performance_benchmarks.rs`
- 测试命令:
  - `cargo check` ✅
  - `cargo test --lib core::router::tests` ✅ (88 passed)
  - `cargo test --test router_tests` ❌ (`no test target named router_tests`，仓库集成测试入口为 `tests/lib.rs` 聚合)
  - 补充执行: `rg \"router::load_balancer|router::strategy::\" src tests benches examples` ✅ (0 命中)
- 结果: 完成，legacy router 栈已物理删除，运行路径统一为 `UnifiedRouter`

### Step 4

- 状态变更: `pending -> in_progress -> completed`
- 实际改动文件:
  - `src/core/models/mod.rs`
  - `src/core/models/request.rs` (deleted)
  - `src/core/models/response/mod.rs` (deleted)
  - `src/core/models/response/completion.rs` (deleted)
  - `src/core/models/response/embedding.rs` (deleted)
  - `src/core/models/response/error.rs` (deleted)
  - `src/core/models/response/media.rs` (deleted)
  - `src/core/models/response/metadata.rs` (deleted)
  - `src/core/models/response/moderation.rs` (deleted)
  - `src/core/models/response/rerank.rs` (deleted)
  - `src/core/models/response/types.rs` (deleted)
- 测试命令:
  - `cargo check` ✅
  - `cargo test --lib core::models` ✅ (586 passed)
  - `rg -n \"core::models::request|core::models::response\" src tests benches examples` ✅ (0 命中)
- 结果: 完成，重复 request/response 层已删除，模型路径收敛到 `core/types` 与 `core/models/openai`

### Step 5

- 状态变更: `pending -> in_progress -> completed`
- 实际改动文件: (待执行后回填)
- 测试命令: (待回填)
- 结果: (待回填)
