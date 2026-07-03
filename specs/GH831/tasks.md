# Task Plan

## Linked Issue

GH-831 / #831

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## 实现任务

- [ ] `SP831-T1` Owner: coordinator. Done when: `specs/GH831/product.md`, `tech.md`, `tasks.md` exist and pass SpecRail packet validation. Verify: `SPEC_RAIL=/path/to/specrail; python3 "$SPEC_RAIL/checks/check_workflow.py" --repo "$SPEC_RAIL" --spec-dir "$PWD/specs/GH831"`.
- [ ] `SP831-T2` Owner: maintainer. Done when: #831 上确认默认 fail-closed 与配置命名（`unpriced_model_policy`），SpecRail human gate `spec_approval` 通过. Verify: #831 issue thread 中维护者的明确批复。
- [ ] `SP831-T3` Owner: coordinator. Done when: pricing service 暴露 usage-aware dry-run/estimate 能力，输入为 `PricingUsage`（包含 audio/image tokens、`output_image_pricing_keys` 等），而不是只判断 provider/model 存在. Verify: `cargo test core::pricing_service --lib --all-features`；新增缺少 image/audio 价格时 dry-run 失败的单测。
- [ ] `SP831-T4` Owner: coordinator. Done when: `unpriced_model_policy` / `unpriced_fallback_cost_per_1k_tokens` 配置模型、校验与默认值（reject）落地，并写明 `pricing.allow_degraded` 只控制 pricing source 初始加载、请求期以 `unpriced_model_policy` 为准. Verify: `cargo test config --lib --all-features`; `rg -n "unpriced_model_policy|allow_degraded" src/config docs CHANGELOG.md`.
- [ ] `SP831-T10` Owner: coordinator. Done when: `KeyManager::record_usage` 或等价写入路径接收显式 `UsageRecord`，storage/in-memory 更新 `KeyUsageStats` 的 `unpriced_requests`、`unpriced_tokens`、`unpriced_cost`、`last_unpriced_at`，读 API 返回这些字段；如果存在明细 spend record/table，每条记录也持久化 `unpriced` 与 `pricing_policy`. Verify: `cargo test core::keys auth::api_key server::routes::keys --all-features`; `rg -n "unpriced_requests|UsageRecord|pricing_policy" src`.
- [ ] `SP831-T5` Owner: coordinator. Done when: router selection 或执行层先排除不可定价 deployment 候选，reject 策略下只有所有候选都不可定价才返回 4xx OpenAI 错误形状且不发往 provider. Verify: `cargo test --all-features spend router`; 新增单测覆盖“未定价候选 + 可定价候选 → 选择可定价候选”和最终 reject 分支。
- [ ] `SP831-T6` Owner: coordinator. Done when: `src/server/routes/ai/spend.rs:532-574` 与 `src/server/routes/ai/spend/pricing.rs:186` 的 pricing-Err 分支改为共享结算辅助函数；allow 策略下 `unpriced_fallback_cost_per_1k_tokens` 按 usage 缩放，有 usage 必结算（不 drop reservation）且 spend/usage 记录带 `unpriced=true`. Verify: `rg -n "unwrap_or\(0.0\)" src/server/routes/ai/spend.rs src/server/routes/ai/spend/pricing.rs` 无未标记残留；新增单测覆盖 allow/usage-scaled fallback/settle/退款分支。
- [ ] `SP831-T7` Owner: coordinator. Done when: metric `gateway_unpriced_spend_total{provider,model_bucket,policy,outcome}` 与 error 日志字段就绪，reject preflight、routing candidate exclusion、settlement fallback 三条路径都记录，且 `model_bucket` 不使用任意请求 model. Verify: `cargo test --all-features metrics`; 手动 `curl /metrics` 观测 label 有界。
- [ ] `SP831-T8` Owner: verification owner. Done when: 全量回归、格式、lint、PR guard 通过，CHANGELOG 记录 breaking-behavior，并说明旧 `pricing.allow_degraded=true` 部署若要继续放行需显式设置 `unpriced_model_policy=allow_unpriced`. Verify: `cargo fmt --all -- --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-features`; `bash scripts/guards/check_pr_scope.sh`.

## 并行拆分

- SP831-T3 与 SP831-T4 可并行（文件不相交：`src/core/pricing_service/` vs `src/config/models/`）。
- SP831-T5 依赖 T3+T4；SP831-T6 依赖 T4 与 T10；SP831-T7 依赖 T5/T6 的 outcome 枚举；SP831-T8 收尾。
- SP831-T5 与 SP831-T6 必须同一 PR 落地，避免出现「预留已拒绝但结算仍退款」的中间态不一致。

## 验证

- [ ] `SP831-T9` Owner: verification owner. Done when: 复现测试证明默认配置下未定价模型请求被拒且预算不变、allow 配置下预留被结算且记录带标记、fallback 按 usage 缩放、可定价 deployment 优先于未定价候选. Verify: `cargo test --all-features spend budget router -- --nocapture`（聚焦模块运行 <60s）。

## Handoff Notes

- 默认 fail-closed 是产品决策（行为收紧），实现必须等 SP831-T2 的维护者批复。
- 实现前先 `rg -n "unwrap_or\(0.0\)" src/server src/core/cost` 画出调用图，确认没有第三处同模式路径。
- 不要把 `unpriced_fallback_cost_per_1k_tokens` 当固定每请求价格；必须乘以 usage 单位并除以 1000。
- Prometheus 不能把原始请求 model 作为无界 label；原始 model 只写结构化日志。
- 与 #840（reserve→call→settle 编排抽象）的先后关系：先修语义（本 issue），后做抽象迁移；迁移时以本 spec 的 Behavior Invariants 为回归基线。
