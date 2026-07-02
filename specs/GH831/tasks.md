# Task Plan

## Linked Issue

GH-831 / #831

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## 实现任务

- [ ] `SP831-T1` Owner: coordinator. Done when: `specs/GH831/product.md`, `tech.md`, `tasks.md` exist and pass SpecRail packet validation. Verify: `SPEC_RAIL=/path/to/specrail; python3 "$SPEC_RAIL/checks/check_workflow.py" --repo "$SPEC_RAIL" --spec-dir "$PWD/specs/GH831"`.
- [ ] `SP831-T2` Owner: maintainer. Done when: #831 上确认默认 fail-closed 与配置命名（`unpriced_model_policy`），SpecRail human gate `spec_approval` 通过. Verify: #831 issue thread 中维护者的明确批复。
- [ ] `SP831-T3` Owner: coordinator. Done when: pricing service 暴露 `can_price(provider, model)` 等价能力并有单测. Verify: `cargo test core::pricing_service --lib --all-features`.
- [ ] `SP831-T4` Owner: coordinator. Done when: `unpriced_model_policy` / `unpriced_fallback_cost_per_1k_tokens` 配置模型、校验与默认值（reject）落地. Verify: `cargo test config --lib --all-features`; `rg -n "unpriced_model_policy" src/config`.
- [ ] `SP831-T5` Owner: coordinator. Done when: 预算预留前接入 can_price gate，reject 策略下未定价请求返回 4xx OpenAI 错误形状且不发往 provider. Verify: `cargo test --all-features spend`; 新增单测覆盖 reject 分支。
- [ ] `SP831-T6` Owner: coordinator. Done when: `src/server/routes/ai/spend.rs:532-574` 与 `src/server/routes/ai/spend/pricing.rs:186` 的 pricing-Err 分支改为共享结算辅助函数；allow 策略下有 usage 必结算（不退款）且 spend 记录带 unpriced 标记. Verify: `rg -n "unwrap_or\(0.0\)" src/server/routes/ai/spend.rs src/server/routes/ai/spend/pricing.rs` 无残留；新增单测覆盖 allow/settle/退款分支。
- [ ] `SP831-T7` Owner: coordinator. Done when: metric `gateway_unpriced_spend_total{provider,model,policy}` 与 error 日志字段就绪. Verify: `cargo test --all-features metrics`; 手动 `curl /metrics` 观测。
- [ ] `SP831-T8` Owner: verification owner. Done when: 全量回归、格式、lint、PR guard 通过，CHANGELOG 记录 breaking-behavior. Verify: `cargo fmt --all -- --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-features`; `bash scripts/guards/check_pr_scope.sh`.

## 并行拆分

- SP831-T3 与 SP831-T4 可并行（文件不相交：`src/core/pricing_service/` vs `src/config/models/`）。
- SP831-T5 依赖 T3+T4；SP831-T6 依赖 T4；SP831-T7 独立；SP831-T8 收尾。
- SP831-T5 与 SP831-T6 必须同一 PR 落地，避免出现「预留已拒绝但结算仍退款」的中间态不一致。

## 验证

- [ ] `SP831-T9` Owner: verification owner. Done when: 复现测试证明默认配置下未定价模型请求被拒且预算不变、allow 配置下预留被结算且记录带标记. Verify: `cargo test --all-features spend budget -- --nocapture`（聚焦模块运行 <60s）。

## Handoff Notes

- 默认 fail-closed 是产品决策（行为收紧），实现必须等 SP831-T2 的维护者批复。
- 实现前先 `rg -n "unwrap_or\(0.0\)" src/server src/core/cost` 画出调用图，确认没有第三处同模式路径。
- 与 #840（reserve→call→settle 编排抽象）的先后关系：先修语义（本 issue），后做抽象迁移；迁移时以本 spec 的 Behavior Invariants 为回归基线。
