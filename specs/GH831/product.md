# Product Spec

## Linked Issue

GH-831 / #831

## 用户问题

在 `origin/main@c47596a4`，任何 pricing catalog 未覆盖的 provider/model 组合的请求在结算层被记为 $0：
预算预留被退回而非结算，per-key spend 记为 0.0。对运营多租户网关的管理员来说，这意味着预算限制
（`UnifiedBudgetLimits`、per-key budget）对未定价模型完全失效，计费口径存在漏洞；对付费租户来说，
花费统计不可信。

这是 money path 上的静默降级：请求成功返回，但计费与预算控制悄悄失效，只留下一条 error 日志。

## 目标

- pricing 计算失败时，预算与计费不再被静默绕过。
- 管理员可以通过配置显式选择失败策略，默认 fail-closed。
- 无论选择哪种策略，行为都可观测（error 日志 + metric + spend 记录标记）。

## 非目标

- 不扩充 pricing catalog 的模型覆盖（catalog 内容更新是常规维护）。
- 不重构 pricing/cost 模块结构（归 #519 A-6）。
- 不改动 reserve→call→settle 编排的抽象方式（归 #840；本 spec 只修正结算语义，落点在现有代码路径上）。

## Behavior Invariants

1. 默认配置下，pricing 解析失败的请求在预算预留阶段被拒绝（HTTP 4xx，OpenAI 错误形状），不会到达 provider。
2. 管理员显式配置 `unpriced_model_policy=allow_unpriced` 时，请求放行，但：
   a. 预算预留按配置的保守单价随实际 usage 等比例结算（或 0 但显式记录 `unpriced=true`），绝不静默退回；
   b. per-key usage/spend 读模型携带可查询的 unpriced 字段或聚合计数，而不是与正常 $0 混同。
3. 结算路径中「预留被退回」只发生在请求失败（未产生 usage）的场景；只要 provider 返回了 usage，预留必须被结算。
4. 每次 unpriced 拒绝或结算触发 error 级日志与专用 metric；metric 的模型维度必须有界，不能把任意请求 model 直接作为 Prometheus label。
5. 已定价模型的现有计费行为完全不变。
6. 当同一个用户请求 model 有多个候选 deployment 时，默认拒绝策略先跳过不可定价候选并尝试可定价候选；只有所有候选都不可定价时才返回最终 `model_not_priced`。

## 验收标准

- [ ] 复现测试：未定价模型 + 带预算 key，默认配置下请求被拒绝且预算不变。
- [ ] 复现测试：开启放行配置后，usage 产生时预留被结算（非退回），spend 记录带 unpriced 标记。
- [ ] 复现测试：配置非 0 `unpriced_fallback_cost_per_1k_tokens` 时，结算金额随 token/image/audio usage 缩放，不是固定每请求金额。
- [ ] 复现测试：默认拒绝路径在返回 4xx 前记录 error 日志并增加 unpriced metric。
- [ ] 复现测试：存在一个未定价候选 deployment 和一个可定价候选 deployment 时，请求路由到可定价候选而不是提前失败。
- [ ] `spend.rs` 与 `spend/pricing.rs` 两处同模式路径行为一致。
- [ ] metric 可在 `/metrics` 观测到。

## 边界情况

- 流式请求中途才拿到 usage：settle 时 pricing 失败同样适用上述 invariants。
- pricing 服务短暂不可用 vs 模型确实不在 catalog：两者对结算层等价（都是 `Err`），策略一致。
- 预留成功但 settle 时才发现 unpriced（价格数据在请求期间被热更新移除）：不能退回；`reject` 策略按既有预留金额或 tech spec fallback 规则结算并打标，`allow_unpriced` 按 usage-scaled fallback 或 0 结算并打标。

## 发布说明

默认行为从「未定价放行且免费」变为「未定价拒绝」，属于行为收紧，需要在 CHANGELOG 标注 breaking-behavior，
并说明恢复旧行为的配置项。
