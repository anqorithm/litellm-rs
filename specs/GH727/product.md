# Product Spec

## Linked Issue

GH-727 / #727

## 用户问题

`origin/main@fec54a56` 仍有 2 个 tracked Rust 文件超过 U-16 的 800 行硬上限。当前最大文件之一是
`src/utils/sync/versioned_map.rs`，它是一个 803 行 optimistic-locking map utility；生产代码承载 `VersionedMap<K, V>`、`VersionedEntry<V>` 和 `VersionError`，超限主要来自 inline unit tests。

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

- 拆分 `src/utils/sync/versioned_map.rs`，它当前 803 行，是 #727 当前最大的 tracked Rust 文件之一。
- 保留 `src/utils/sync/versioned_map.rs` 作为 `VersionedMap<K, V>` production implementation owner。
- 将原 inline tests 移动到 `src/utils/sync/versioned_map_tests.rs`，通过 path-backed child test module 继续访问 parent module imports。
- 不改变 DashMap/AtomicU64 storage strategy, public method signatures, Clone/Default impls, compare-and-swap semantics, retry fallback behavior, global version behavior, or concurrency test expectations。
- 所有新增或修改后的 Rust 文件低于 800 行。

## 非目标

- 不修改 `src/utils/sync/mod.rs` public re-exports, `ConcurrentMap`, `ConcurrentVec`, `AtomicValue`, or unrelated sync container behavior。
- 不改变 insert/get/get_versioned/get_version/compare_and_swap/update_with_retry/get_or_insert/entries semantics or thread concurrency assertions。
- 不在本 PR 中处理其余 1 个大文件。
- 不关闭 #727。

## Behavior Invariants

1. `src/utils/sync/versioned_map.rs` keeps `VersionError`, `VersionedEntry<V>`, `VersionedMap<K, V>`, and their public implementations.
2. Parent module delegates tests with `#[path = "versioned_map_tests.rs"] mod tests;`.
3. Child test module continues to access the production type and parent imports through `super::*`.
4. Basic map operations, optimistic locking, retry fallback, global version increments, clone sharing, and concurrent insert/update behavior stay unchanged.
5. Tests move without assertion or fixture changes.
6. No sync module public re-export or non-`VersionedMap` sync container behavior is changed.
7. Every touched Rust file must be below U-16's 800-line ceiling.
8. `cargo test utils::sync::versioned_map --lib --all-features` must pass.

## 验收标准

- [ ] `src/utils/sync/versioned_map.rs` delegates inline tests to a path-backed child module。
- [ ] Original VersionedMap tests move without assertion changes。
- [ ] Production VersionedMap storage, methods, errors, and sync module re-export paths stay unchanged。
- [ ] All touched VersionedMap files are below U-16's 800-line ceiling。
- [ ] Focused VersionedMap test suite 通过。
- [ ] `cargo fmt --all -- --check`、`cargo check --lib --all-features`、`cargo check --all-features --locked` 和 `cargo check` 通过。
- [ ] PR body 明确该 PR 是 #727 的 partial tranche，使用 `Refs #727`，不自动关闭 tracker issue。

## 发布说明

No runtime behavior change. This is a VersionedMap unit-test extraction for U-16 compliance.
