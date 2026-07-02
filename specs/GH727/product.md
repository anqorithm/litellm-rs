# Product Spec

## Linked Issue

GH-727 / #727

## 用户问题

`origin/main@1bbe63fd` 只剩 1 个 tracked Rust 文件超过 U-16 的 800 行硬上限：
`src/core/user_management/types.rs`，当前 803 行。该文件的生产代码只定义
`User`、`Team`、`Organization` 和 `TeamMember` 四个 public entity DTO；超限来自
inline unit tests，而不是类型职责过载。

本轮是 #727 的最终 tranche。目标是在不改变 user-management public API、不新增 facade
层级、不改变 serde 字段契约的前提下，把最后一个 over-800 文件拆到 U-16 范围内，并在
最终扫描确认没有剩余 over-800 Rust 文件后关闭 #727。

## 全量目标

- 当前 tracked Rust 文件全部低于 U-16 的 800 行硬上限。
- 每个 tranche 只拥有一个文件或一个紧密文件家族。
- 拆分沿现有架构边界进行：测试按行为域外置，public entity types 保持原 module path。
- 对 public API 类型文件优先使用 test extraction；只有生产类型本身过载时才引入 facade + `pub use`。
- 不改变 runtime behavior、serde shape、role/settings imports、user-management re-export path。
- 最终 PR 在本地和 GitHub gate 均通过后可使用 closing keyword 关闭 #727。

## 解耦分层

| Lane | 文件类型 | 代表文件 | 拆分策略 |
| --- | --- | --- | --- |
| A | Test-only suites | provider/router/utils/integration tests | 保持原测试断言和 focused test coverage，按行为域拆成 child test modules。 |
| B | Public type facades | SDK, analytics, monitoring, observability, audio, config, user/key/cache types | 若超限由 inline tests 造成，先外置 tests；否则用 facade + `pub use` 保持路径兼容。 |
| C | Runtime orchestrators | OpenTelemetry, OAuth session, request validator | 抽出 request mapping、response mapping、operation handlers、storage helpers 或 error mapper。 |
| D | Shared utilities | config/net/sync helpers | 按功能域拆 util module，避免新增全局 prelude 或 Any-like public API。 |
| E | Closure scan | all tracked Rust files | 全量 line-count 扫描为 0 后，最终 PR 可关闭 #727。 |

## 本 tranche 目标

- 拆分 `src/core/user_management/types.rs`，它当前 803 行，是 #727 最后一个 over-800 tracked Rust 文件。
- 保留 `src/core/user_management/types.rs` 作为 user-management public entity type owner。
- 将原 inline tests 移动到 `src/core/user_management/types_tests.rs`，通过 path-backed child test module 继续访问 parent module imports。
- 不改变 `User`、`Team`、`Organization`、`TeamMember` fields, derives, serde compatibility, role/settings type references, or `src/core/user_management/mod.rs` re-exports。
- 所有新增或修改后的 Rust 文件低于 800 行。
- 全量 tracked Rust 扫描不再返回任何 over-800 文件。

## 非目标

- 不拆分生产 entity structs 到多个 public modules。
- 不修改 `src/core/user_management/mod.rs` public re-exports。
- 不改变 roles/settings/manager/team_ops/user_ops 行为。
- 不改变任何测试断言、fixture 字段值或 serde JSON shape。
- 不引入新的 user-management facade、trait、persistence layer 或 Any-like public API。

## Behavior Invariants

1. `src/core/user_management/types.rs` keeps the four public entity structs and their existing derives/fields.
2. Parent module delegates tests with `#[path = "types_tests.rs"] mod tests;`.
3. Child test module continues to access production types and imports through `super::*`.
4. User/team/organization/team-member construction, clone/debug/serde, role coverage, relationship, budget, active-member, and deserialization assertions stay unchanged.
5. `src/core/user_management/mod.rs` continues to re-export `Organization`, `Team`, `TeamMember`, and `User` from `types`.
6. Every touched Rust file must be below U-16's 800-line ceiling.
7. Final tracked Rust over-800 scan must be empty.
8. `cargo test core::user_management::types --lib --all-features` must pass.

## 验收标准

- [ ] `src/core/user_management/types.rs` delegates inline tests to a path-backed child module。
- [ ] Original user-management type tests move without assertion changes。
- [ ] Public entity fields, derives, serde behavior, and `user_management` re-export paths stay unchanged。
- [ ] All touched user-management type files are below U-16's 800-line ceiling。
- [ ] Focused user-management type test suite 通过。
- [ ] Full tracked Rust over-800 scan returns no files。
- [ ] `cargo fmt --all -- --check`、`cargo check --lib --all-features`、`cargo check --all-features --locked` 和 `cargo check` 通过。
- [ ] PR body 明确该 PR is the final #727 tranche and may close #727 after clean local and GitHub gates。

## 发布说明

No runtime behavior change. This is the final GH727 user-management type test extraction and closure scan for U-16 compliance.
