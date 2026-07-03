# Task Plan

## Linked Issue

GH-840 / #840

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## 实现任务

- [ ] `SP840-T1` Owner: coordinator. Done when: `specs/GH840/` 三件套通过 SpecRail packet validation. Verify: `SPEC_RAIL=/path/to/specrail; python3 "$SPEC_RAIL/checks/check_workflow.py" --repo "$SPEC_RAIL" --spec-dir "$PWD/specs/GH840"`.
- [ ] `SP840-T2` Owner: maintainer. Done when: #840 批复抽象形态（BudgetedExecutor + SettlementMode + SettledStream）与迁移顺序（SpecRail human gate `spec_approval`），并确认与 #831 的先后关系. Verify: #840 issue thread 明确批复。
- [ ] `SP840-T3` Owner: coordinator. Done when: `BudgetedExecutor` + `SettlementMode::{Metered,AvailabilityOnly,KeyReservationThenPostSuccessRecord}` + 有序 router candidates + 重试兼容 `SelectedDeploymentContext` callback + per-attempt `PreCallCharge` resolver 合入，四分支（预算不足/成功结算/失败退回/settle 失败）单测覆盖，`AvailabilityOnly` 证明不记账，`AppState.budgeted` 就位；旧预算字段可暂留到端点迁移完成后的 T8 收尾，避免基础设施 PR 超范围迁移端点. Verify: `cargo test server::routes::ai::budgeted --lib --all-features`。
- [ ] `SP840-T4` Owner: coordinator. Done when: `SettledStream` / 流响应驱动合入，结算使用显式 async finalization 而不是 `Drop`；覆盖 usage 中段、chat/completions 正常结束无 usage 但有上游输出、chat/completions 空成功流扣预留、native Gemini 空成功流退款、Gemini 无 usage metadata 的 upstream error 退款、客户端断开、预上游输出错误退回、错误终止，并验证当前应记录的 `StreamingDeploymentLease::finish_success` / `finish_failure` 仍发生. Verify: `cargo test --all-features settled_stream`。
- [ ] `SP840-T5` Owner: coordinator. Done when: chat 非 stream + stream 迁移完成，现有 chat 测试全绿且流式 settle 时机与迁移前逐行对照记录进 PR body. Verify: `cargo test --all-features chat`; PR body 对照表。
- [ ] `SP840-T6` Owner: coordinator. Done when: completions / embeddings / images / audio×3 迁移完成（可拆多 PR，每 PR 一个端点家族）；image generation/audio 使用 selected-deployment 派生的 per-attempt `PreCallCharge`，image edit/variation proxy 显式豁免 selected-deployment 重新定价并保持现有 request-level OpenAI pricing identity 预计算 cost +「API key 预留 + upstream status 成功后、body conversion 前 provider/model spend」而非 provider/model 预调用预留；chat/embeddings cache hit 仍在 `run` 前返回且不新增预算副作用. Verify: 各端点聚焦测试 + `cargo test --all-features`。
- [ ] `SP840-T7` Owner: coordinator. Done when: gemini / responses_stream / moderations / rerank / fine_tuning / batches 迁移完成或逐文件写明保持现状的豁免理由；moderations、rerank、fine_tuning、batches 若当前仅做可用性检查，则用 `AvailabilityOnly` 显式声明并证明不新增 spend/key usage. Verify: 各端点聚焦测试；`rg -n "AvailabilityOnly" src/server/routes/ai/{moderations,rerank,fine_tuning,batches}.rs` 或 PR body 中逐文件豁免说明。
- [ ] `SP840-T8` Owner: verification owner. Done when: 样板清零——直接预算字段访问 guard 覆盖 `state\.(budget_limits|pricing|key_manager|budget_manager)\b`（不只 `.clone`），AI route 迁移完成后旧 AppState 预算字段已隐藏/降可见性/内部 wrapper 化或以 accessor 限制预算用途进入 budgeted/spend；`key_manager` 在 `/v1/keys` 等非预算管理路由的合法访问不在本 guard 范围内；兄弟 route 不能 import/call `execution::execute_*`; 允许项仅限 budgeted/spend 内部与测试中明确列出的 helper. Verify: `rg -n "state\.(budget_limits|pricing|key_manager|budget_manager)\b" src/server/routes/ai --glob '!budgeted.rs' --glob '!budgeted/**' --glob '!spend.rs' --glob '!spend/**' --glob '!*_tests.rs'`; `rg -n "(execution::execute_|execute_with_selected_deployment\\(|execute_stream_with_selected_deployment\\()" src/server/routes/ai --glob '!budgeted.rs' --glob '!budgeted/**' --glob '!execution.rs' --glob '!*_tests.rs'`；输出进收尾 PR body。

## 并行拆分

- SP840-T3 与 SP840-T4 可并行（新文件，互不相交）。
- SP840-T6/T7 各端点家族 PR 文件不相交可并行（W-14）；SP840-T5 先行（最复杂样本验证抽象设计）。
- 强依赖：#831 的语义修复先合并，本 issue 迁移以 #831 后的行为为基线（避免把旧缺口固化进抽象）。

## 验证

- [ ] `SP840-T9` Owner: verification owner. Done when: 带预算 key 的 chat/embeddings/images 端到端回归（预算不足拒绝、正常扣费、失败退回）在收尾 PR 记录本会话命令输出. Verify: `cargo test --all-features` + 集成测试输出（W-16）。

## Handoff Notes

- 先后关系是硬约束：#831（语义）→ 本 issue（结构）。顺序颠倒会把「pricing 失败静默退款」复制进新抽象。
- 迁移期间禁止顺手改行为（U-07）；发现疑似 bug 记 issue 不修，保持每 PR 可逐行对照。
- `AppState` 4 个旧字段的收缩评估放在全部端点迁移完成后单独提案（U-01 公开 API 约束）。
