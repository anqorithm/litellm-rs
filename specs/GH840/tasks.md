# Task Plan

## Linked Issue

GH-840 / #840

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## 实现任务

- [ ] `SP840-T1` Owner: coordinator. Done when: `specs/GH840/` 三件套通过 SpecRail packet validation. Verify: `SPEC_RAIL=/path/to/specrail; python3 "$SPEC_RAIL/checks/check_workflow.py" --repo "$SPEC_RAIL" --spec-dir "$PWD/specs/GH840"`.
- [ ] `SP840-T2` Owner: maintainer. Done when: #840 批复抽象形态（BudgetedExecutor + SettlementMode + SettledStream）与迁移顺序（SpecRail human gate `spec_approval`），并确认与 #831 的先后关系. Verify: #840 issue thread 明确批复。
- [ ] `SP840-T3` Owner: coordinator. Done when: `BudgetedExecutor` + `SettlementMode` 合入，四分支（预算不足/成功结算/失败退回/settle 失败）单测覆盖，`AppState.budgeted` 就位. Verify: `cargo test server::routes::ai::budgeted --lib --all-features`。
- [ ] `SP840-T4` Owner: coordinator. Done when: `SettledStream`（RAII 结算守卫）合入，三场景（usage 中段/无 usage 断开/错误终止）单测覆盖. Verify: `cargo test --all-features settled_stream`。
- [ ] `SP840-T5` Owner: coordinator. Done when: chat 非 stream + stream 迁移完成，现有 chat 测试全绿且流式 settle 时机与迁移前逐行对照记录进 PR body. Verify: `cargo test --all-features chat`; PR body 对照表。
- [ ] `SP840-T6` Owner: coordinator. Done when: completions / embeddings / images / audio×3 迁移完成（可拆多 PR，每 PR 一个端点家族）. Verify: 各端点聚焦测试 + `cargo test --all-features`。
- [ ] `SP840-T7` Owner: coordinator. Done when: gemini / responses_stream / moderations / rerank 迁移完成（moderations、rerank 用 `RecordOnly` 模式显式声明）. Verify: 各端点聚焦测试；`rg -n "RecordOnly" src/server/routes/ai/{moderations,rerank}.rs`。
- [ ] `SP840-T8` Owner: verification owner. Done when: 样板清零——`rg "state\.(budget_limits|pricing|key_manager|budget_manager)\.clone" src/server/routes/ai` 除 budgeted.rs 外零命中；裸执行函数不再 pub. Verify: 上述 `rg` 输出进收尾 PR body。

## 并行拆分

- SP840-T3 与 SP840-T4 可并行（新文件，互不相交）。
- SP840-T6 各端点家族 PR 文件不相交可并行（W-14）；SP840-T5 先行（最复杂样本验证抽象设计）。
- 强依赖：#831 的语义修复先合并，本 issue 迁移以 #831 后的行为为基线（避免把旧缺口固化进抽象）。

## 验证

- [ ] `SP840-T9` Owner: verification owner. Done when: 带预算 key 的 chat/embeddings/images 端到端回归（预算不足拒绝、正常扣费、失败退回）在收尾 PR 记录本会话命令输出. Verify: `cargo test --all-features` + 集成测试输出（W-16）。

## Handoff Notes

- 先后关系是硬约束：#831（语义）→ 本 issue（结构）。顺序颠倒会把「pricing 失败静默退款」复制进新抽象。
- 迁移期间禁止顺手改行为（U-07）；发现疑似 bug 记 issue 不修，保持每 PR 可逐行对照。
- `AppState` 4 个旧字段的收缩评估放在全部端点迁移完成后单独提案（U-01 公开 API 约束）。
