# Product Spec

## Linked Issue

GH-727 / #727

## 用户问题

`origin/main@804aff95` 仍有 4 个 tracked Rust 文件超过 U-16 的 800 行硬上限。当前最大文件是
`src/core/observability/metrics.rs`，它是一个 808 行 observability metrics module；生产代码只到 metrics collector、Prometheus export、DataDog/Otel config helpers，超限主要来自 inline unit tests。

本轮目标继续执行完整的大文件解耦计划：每个 PR 仍然小而可审，但所有 tranche 都必须服从同一套
架构边界，避免制造新的耦合、重导出混乱或行为漂移。

## 全量目标

- 把当前 over-800 Rust 文件逐步拆到 U-16 范围内。
- 每个 tranche 只拥有一个文件或一个紧密文件家族。
- 拆分必须沿现有架构边界进行：测试按行为域拆、类型按领域 DTO/状态/配置拆、运行时代码按
  provider/route/repository/validator/adapter 职责拆。
- 对 public API 类型文件使用 facade + `pub use` 保持现有导入路径兼容。
- 对运行时代码保留现有错误语义；不得用 warning、fallback 或 silently ignore 代替错误。
- #727 只在最后一次全量扫描确认没有 over-800 Rust 文件后才允许使用 closing keyword。

## 解耦分层

| Lane | 文件类型 | 代表文件 | 拆分策略 |
| --- | --- | --- | --- |
| A | Test-only suites | `src/utils/data/utils/tests.rs`, router tests, utils/event tests, provider test files, integration route tests | 保持原测试断言和 focused test coverage，按行为域拆成 child test modules。 |
| B | Public type facades | SDK, analytics, monitoring, observability, audio, config, user/key/cache types | 建立 `types/` 子模块或外置 tests；root 继续保留原有 public type paths。 |
| C | Runtime orchestrators | OpenTelemetry, OAuth session, request validator, provider modules | 抽出 request mapping、response mapping、operation handlers、storage helpers 或 error mapper，保留外层入口和 trait surface。 |
| D | Shared utilities | config/net/sync helpers | 按功能域拆 util module，避免新增全局 prelude 或 Any-like public API。 |

## 本 tranche 目标

- 拆分 `src/core/observability/metrics.rs`，它当前 808 行，是 #727 当前最大的 tracked Rust 文件。
- 保留 `src/core/observability/metrics.rs` 作为 production metrics collector、Prometheus metrics struct、DataDog client config 和 OpenTelemetry exporter owner。
- 将原 inline tests 移动到 `src/core/observability/metrics_tests.rs`，通过 path-backed child test module 继续访问私有 fields。
- 不改变 request/error/token/cost/cache/provider-health recording, Prometheus export formatting, DataDog no-op send behavior, histogram duration recording, or edge-case expectations。
- 所有新增或修改后的 Rust 文件低于 800 行。

## 非目标

- 不修改 metrics production API, public re-exports, histogram implementation, TokenUsage type, outbound HTTP client wiring, DataDog payload implementation, or OpenTelemetry behavior。
- 不改变 metric names, labels, cache counters, provider-health gauges, token/cost counters, duration histogram assertions, or existing no-network DataDog test semantics。
- 不在本 PR 中处理其余 3 个大文件。
- 不关闭 #727。

## Behavior Invariants

1. `src/core/observability/metrics.rs` keeps `PrometheusMetrics`, `DataDogClient`, `OtelExporter`, and `MetricsCollector` definitions and public method signatures.
2. Parent module delegates tests with `#[path = "metrics_tests.rs"] mod tests;`.
3. Child test module continues to access private metrics collector fields through `super::*`.
4. Request recording, cache recording, provider health, Prometheus export, DataDog send, duration, and edge-case behavior stay unchanged.
5. Tests move without assertion or fixture changes.
6. No observability production behavior, public re-export, histogram, or TokenUsage behavior is changed.
7. Every touched Rust file must be below U-16's 800-line ceiling.
8. `cargo test core::observability::metrics --lib --all-features` must pass.

## 验收标准

- [ ] `src/core/observability/metrics.rs` delegates inline tests to a path-backed child module。
- [ ] Original metrics tests move without assertion changes。
- [ ] Production metrics structs, collector methods, and public re-export paths stay unchanged。
- [ ] All touched observability metrics files are below U-16's 800-line ceiling。
- [ ] Focused observability metrics test suite 通过。
- [ ] `cargo fmt --all -- --check`、`cargo check --lib --all-features`、`cargo check --all-features --locked` 和 `cargo check` 通过。
- [ ] PR body 明确该 PR 是 #727 的 partial tranche，使用 `Refs #727`，不自动关闭 tracker issue。

## 发布说明

No runtime behavior change. This is an observability metrics unit-test extraction for U-16 compliance.
