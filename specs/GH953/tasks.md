# GH953 任务计划

- [x] `SP953-T1` Owner: auth worker. Dependencies: none. Done when: `AuthMethod` 的 `Debug` 与认证入口日志不包含凭证字节。Verify: `cargo test auth_method --lib --all-features`。
- [x] `SP953-T2` Owner: auth worker. Dependencies: `SP953-T1`. Done when: 普通/详细 API-key 验证共享 owner 判定，missing/inactive owner 均被拒绝，基础设施错误仍为 `Err`。Verify: `cargo test api_key --lib --all-features`。
- [x] `SP953-T3` Owner: auth worker. Dependencies: `SP953-T2`. Done when: 认证只读取数据库权威状态，缓存失效失败不影响已撤销 key 的拒绝结果。Verify: 不调用缓存失效、直接撤销数据库后的双入口回归测试。
- [x] `SP953-T4` Owner: auth worker. Dependencies: `SP953-T2`. Done when: 客户端能区分 invalid key 与 verification unavailable，且看不到底层错误详情。Verify: 聚焦 auth 测试与代码检查。
- [x] `SP953-T5` Owner: verification owner. Dependencies: `SP953-T1..T4`. Done when: 格式、编译、clippy、全量测试和 PR scope guard 通过。Verify: `cargo fmt --all -- --check`; `cargo check --all-features`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-features`; `bash scripts/guards/check_pr_scope.sh`。

## Handoff

- `pr_kind: mixed_impl`
- `completion_mode: final`
- PR 使用 `Refs #953`；canonical-user 删除的 FK 策略需要维护者决定，不能在当前 tranche 中声称完整关闭 issue。
