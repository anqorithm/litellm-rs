# Task Plan

## Linked Issue

Private advisory: `GHSA-96qr-wjxq-r4xj`

## Execution Constraints

- 实现分支必须从包含本 Spec 的私有基线创建，并保留其 `origin/main` 基底
  `0baa92798d15630edc0b6abd65646b25e49ca23c`。
- Spec PR 与 Impl PR 分离；实现 PR 不得混入无关重构或依赖升级。
- 该事项留在临时私有 fork，不创建公开 Issue/PR，不在日志或公开分支披露安全细节。
- 不自动合并到 `main`。

## Implementation Tasks

### SPGHSA96QR-T1 — 收敛 compatibility adapter

- Owner: implementation agent
- Dependencies: approved `product.md` and `tech.md`
- Covers: B-001, B-003, B-004, B-005
- Work:
  - 保留公开函数签名、URL parse 与 HTTP/HTTPS scheme 契约。
  - 委托统一 `ProviderEndpointAccess::PublicOnly` guard 并映射带 context 的错误。
  - 删除本文件重复 hostname/IP/DNS 分类实现与不再使用的 imports。
- Done when:
  - `src/config/validation/ssrf.rs` 不再声明 `is_private_or_internal_ip` 或直接使用 `ToSocketAddrs`。
  - 所有失败路径返回 `Err` 且包含调用方 context。
- Verify:
  - `rg -n "is_private_or_internal_ip|ToSocketAddrs" src/config/validation/ssrf.rs` 返回零命中。
  - `cargo test --all-features --locked config::validation::ssrf::tests`

### SPGHSA96QR-T2 — 建立确定性策略回归矩阵

- Owner: implementation agent
- Dependencies: SPGHSA96QR-T1
- Covers: B-002, B-005, B-006
- Work:
  - 在 `src/config/validation/ssrf.rs` 与 `src/config/validation/tests.rs` 中用公网 literal 重写成功用例，
    保留 HTTP/HTTPS、端口、路径和 query 行为覆盖。
  - 添加四个已复现 special-purpose 范围负例。
  - 删除只针对已移除私有 helper 的单元测试，不削弱公开行为断言。
- Done when:
  - 两个 compatibility test 模块的成功路径都不含需要公网 DNS 的 hostname。
  - 四个策略分叉地址均通过公开入口被拒绝。
- Verify:
  - `cargo test --all-features --locked config::validation::ssrf::tests`
  - `cargo test --all-features --locked config::validation::tests::test_ssrf_validation`
  - 用上述 Cargo 命令生成的当前 lib test executable，对两个 filter 各直接连续运行 50 次；避免每轮
    重复链接，但不得减少测试内容或次数。

## Verification Tasks

### SPGHSA96QR-T3 — 完整质量与安全验证

- Owner: implementation agent
- Dependencies: SPGHSA96QR-T1, SPGHSA96QR-T2
- Covers: B-001, B-002, B-003, B-004, B-005, B-006
- Done when:
  - 格式、全 feature 编译、strict Clippy 和全 feature 测试均产生当前 head 的成功证据。
  - `cargo audit` 结果完整记录；若仅存在基线 advisory，必须明确区分而非声称零告警。
- Verify:
  - `cargo fmt --all -- --check`
  - `cargo check --all-targets --all-features --locked`
  - `cargo clippy --all-targets --all-features --locked -- -D warnings`
  - `cargo test --all-features --locked`
  - `cargo audit`

## Handoff

### SPGHSA96QR-T4 — 私有 Impl PR 交付

- Owner: implementation agent
- Dependencies: SPGHSA96QR-T3
- Covers: none — 该任务只封装已验证实现与证据，不新增产品行为。
- Done when:
  - Impl PR 链接私有 advisory 与 Spec PR，说明根因、方案、风险、测试和剩余基线告警。
  - diff 只包含 `src/config/validation/ssrf.rs` 与 `src/config/validation/tests.rs` 的实现/测试变化。
  - PR 保持未合并，等待最终人工合并决策。
- Verify:
  - `git diff --check origin/main...HEAD`
  - `git diff --stat origin/main...HEAD`

## Invariant Coverage Audit

- Product IDs: `B-001, B-002, B-003, B-004, B-005, B-006`
- Task coverage union: `B-001, B-002, B-003, B-004, B-005, B-006`
- Missing: none
