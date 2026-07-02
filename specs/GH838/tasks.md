# Task Plan

## Linked Issue

GH-838 / #838

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## 实现任务

- [ ] `SP838-T1` Owner: coordinator. Done when: `specs/GH838/` 三件套通过 SpecRail packet validation. Verify: `SPEC_RAIL=/path/to/specrail; python3 "$SPEC_RAIL/checks/check_workflow.py" --repo "$SPEC_RAIL" --spec-dir "$PWD/specs/GH838"`.
- [ ] `SP838-T2` Owner: coordinator. Done when: 可达性证据表（每子系统的 `rg` 命中数、依赖、测试规模、churn）作为附录追加到本 spec. Verify: `git diff -- specs/GH838/`; 每子系统一行且附判定命令。
- [ ] `SP838-T3` Owner: maintainer. Done when: 维护者在 #838 批复处置矩阵（wire/remove/experimental-gate），特别是 mcp/a2a/realtime 的产品意向与 guardrails 默认开关（SpecRail human gate `spec_approval`）. Verify: #838 issue thread 中的明确批复。
- [ ] `SP838-T4` Owner: coordinator. Done when: 守护检查合入 CI——`src/core/mod.rs` 顶层 `pub mod` 必须被 server/main 引用、或在豁免清单、或被真 feature gate. Verify: 检查脚本/测试绿色；人为添加未接线模块的负测试验证后移除。
- [ ] `SP838-T5` Owner: coordinator. Done when: wire lane 执行完毕——每子系统一个 PR，含 `GatewayConfig` 字段、启动初始化、中间件/路由挂载、smoke 测试（U-26 三要件）. Verify: 每 PR `cargo test --all-features` + 冒烟请求记录。
- [ ] `SP838-T6` Owner: coordinator. Done when: gate lane 执行完毕——被 gate 模块默认不编译，README 标注 experimental，docs.rs feature 列表同步. Verify: `cargo check --no-default-features --features "metrics,tracing"` 与 `cargo check --all-features` 双向通过。
- [ ] `SP838-T7` Owner: coordinator. Done when: remove lane 执行完毕，`core/mod.rs` 与文档同步清理. Verify: `cargo check --all-features`; `rg -n "MCP Gateway|A2A Protocol" CLAUDE.md README.md` 输出与处置一致。
- [ ] `SP838-T8` Owner: verification owner. Done when: 全量回归通过，README/CLAUDE.md 能力表与实际可达能力一致. Verify: `cargo fmt --all -- --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-features`.

## 并行拆分

- SP838-T2（纯文档）与 SP838-T4（守护检查，只动 CI/测试文件）可并行。
- SP838-T5 各子系统 PR 文件不相交可并行（W-14），但 observability+integrations 因依赖必须同组。
- SP838-T6/T7 依赖 T3 批复；T5 中安全语义子系统（guardrails/ip_access）优先。

## 验证

- [ ] `SP838-T9` Owner: verification owner. Done when: 被 wire 的每个子系统有一条本会话可复现的端到端证据（冒烟请求或 metric 输出）记录在对应 PR body. Verify: PR body 中的命令输出（W-16：本会话证据）。

## Handoff Notes

- 与 #837 的边界：本 issue 只处理 core 子系统层；provider 目录归 #837。两者的守护检查可共享豁免清单机制但分开断言。
- batch 半接线态是最优先消除项：现状「路由存在但持久化被绕过」比完全未接线更误导（用户以为 batch 有持久化）。
- guardrails 若 wire，注意其 `check_output` 每次调用重新编译正则（`src/core/guardrails/prompt_injection.rs:294-303`），接线前先改为预编译（`LazyLock`），否则把性能问题带上热路径。
