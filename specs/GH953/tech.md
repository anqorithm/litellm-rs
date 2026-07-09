# GH953 技术规格：认证凭证与 API-key 生命周期加固

## 当前行为与根因

| 区域 | 当前行为 | 风险 |
|---|---|---|
| `src/auth/types.rs`、`src/auth/system.rs` | `AuthMethod` 派生 `Debug`，认证入口记录完整值 | 日志泄露凭证 |
| `src/auth/api_key/creation.rs` | `verify_key` 不拒绝 missing/inactive owner；详细入口仅拒绝 inactive | 两个 live 入口语义漂移 |
| `src/auth/api_key/creation.rs` | cache hit 直接作为 key 真值 | revoke 后可继续信任旧 active 快照 |
| `src/auth/api_key/management.rs`、key handler | 缓存删除错误只告警 | 缓存若参与认证，成功撤销不能证明状态已收敛 |
| `src/auth/system.rs` | 将底层验证错误拼入公开 `AuthResult.error` | 内部错误泄露，invalid 与 outage 混淆 |

## 设计

1. 手写 `AuthMethod::Debug`，只输出 variant 与固定 `[REDACTED]`，认证日志继续保留方式级可观测性。
2. 在 API-key handler 内建立共享验证函数，统一检查 active、expiry、owner 存在性与 owner active 状态；普通入口由详细入口派生成功/失败结果。
3. API-key 认证直接读取数据库权威状态，不再读取或刷新 Redis snapshot。既有缓存失效保留为 best-effort 兼容性清理；即使 cache delete 失败，旧 active 快照也不会参与授权。
4. 撤销仍以数据库 mutation 成功为准；Redis 故障不把已完成的安全 mutation 伪装成失败，也不影响后续认证结果。
5. `AuthSystem` 服务端记录底层验证错误，公开结果使用固定的“verification unavailable”，与“Invalid API key”区分且不带内部详情。

## 不变范围

- 不修改 API-key hash、TTL、数据库 schema 或 Redis key 格式。
- 不修改 JWT/session 验证流程。
- 不改变未绑定 owner 的 key 语义。

## 映射

| invariant | 实现 | 验证 |
|---|---|---|
| `GH953-B1` | `AuthMethod` 手写 `Debug` | 三个 variant 的 redaction 单测 |
| `GH953-B2/B3` | 共享 key/owner 判定 | inactive/missing owner 集成测试 |
| `GH953-B4` | 认证只读 DB authoritative state | 不调用缓存失效，直接撤销 DB 后验证两种入口均拒绝 |
| `GH953-B5` | 固定公开消息、内部 `error!` | `AuthSystem` 聚焦测试/静态断言 |

## 风险与回滚

- 风险：每次 API-key 验证需要一次权威 DB 查询；这是保证跨实例 revoke 正确性的有意安全取舍，并移除了无效的 Redis GET/SET 往返。
- 回滚：可整体回退本 tranche；无 schema 或数据迁移。

## 验证计划

- `cargo test auth_method --lib --all-features`
- `cargo test api_key --lib --all-features`
- `cargo fmt --all -- --check`
- `cargo check --all-features`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features`
