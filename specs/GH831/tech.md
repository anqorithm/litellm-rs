# Tech Spec

## Linked Issue

GH-831 / #831

## Product Spec

Link to `product.md`.

## Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| Spend settlement | `src/server/routes/ai/spend.rs:532-574` | pricing `Err` → `cost = None` → settle/record_spend 跳过、预留 drop 退回、`record_usage(.., 0.0)` | 主要缺口 |
| Spend pricing helper | `src/server/routes/ai/spend/pricing.rs:186` | 同模式 warn-then-None | 同步修复点 |
| Budget reservation | `src/core/budget/`（`reservation.settle` 调用方在 spend.rs） | 预留 drop 即退款 | invariant 3 的落点 |
| Pre-request check | `ensure_budget_available` / `reserve_*_budget`（chat/completions 等各端点） | 预留时不校验模型是否可定价 | fail-closed 的拦截点 |
| Pricing service | `src/core/pricing_service/service.rs` | `calculate_loaded_usage_cost_for_provider` 返回 `Err` | 需要区分「不可定价」错误类型的来源 |
| Config | `src/config/models/gateway.rs` | `GatewayPricingConfig::allow_degraded` 只控制 pricing source 初始加载失败时是否继续启动 | 新策略必须定义与现有降级开关的优先级 |
| Usage read/write shape | `src/auth/api_key/management.rs`、`src/core/keys/types.rs` | `record_usage` 只接收 tokens/cost；`KeyUsageStats` 只有聚合计数和金额 | 需要可查询的 `unpriced` 读模型 |
| Router selection | `src/core/router/selection.rs`、`src/server/routes/ai/execution.rs` | selection 支持 candidate predicate，但当前 budget/pricing gate 在选中 deployment 后执行 | 需要先排除不可定价候选，再返回最终拒绝 |

## 设计方案

1. **配置**：在 gateway pricing/budget 配置中新增
   `unpriced_model_policy: reject | allow_unpriced`（默认 `reject`），
   以及可选 `unpriced_fallback_cost_per_1k_tokens: Option<f64>`。该 fallback 是每 1k
   usage 单位价格，不能按固定每请求金额使用；结算金额必须为
   `fallback_per_1k * billable_usage_units / 1000.0`。
2. **与 `pricing.allow_degraded` 的优先级**：
   - `GatewayPricingConfig::allow_degraded=true` 只表示 pricing source 初始加载失败时 gateway
     可以继续启动；它不自动允许未定价请求免费通过。
   - 请求期策略以 `unpriced_model_policy` 为准。若 pricing source 因 `allow_degraded=true`
     缺失或过期，`reject` 仍返回 `model_not_priced`，`allow_unpriced` 才允许带标记结算。
   - 需要恢复旧的降级运行方式时，操作者必须同时配置
     `pricing.allow_degraded=true` 与 `unpriced_model_policy=allow_unpriced`；迁移文档必须写明。
3. **usage-aware fail-closed 拦截**：预算预留前增加一次「可定价性」检查，但不能只检查
   provider/model 是否存在。检查输入必须是 endpoint 已有或可估算的 `PricingUsage`
   形状（tokens、audio/image tokens、`output_image_pricing_keys` 等）并调用 pricing service
   的 dry-run/estimate 接口；任何 usage 形状无法估价都视为不可定价。
4. **routing 语义**：在 router selection 阶段使用 candidate predicate 或等价 retryable
   candidate error，把不可定价 deployment 从候选中排除并继续尝试同一请求 model 的其他
   deployment。只有所有健康且有能力的候选都不可定价时，才返回 OpenAI 错误形状 4xx
   （`model_not_priced` 语义），并保证请求不发往 provider。chat / embeddings 等 response-cache
   命中路径也必须在返回 cached response 前执行同一 policy gate，不能因跳过 routing/budget 而绕过 reject。
5. **结算语义修正**：`record_settled_spend`（spend.rs）中 pricing `Err` 分支改为：
   - `allow_unpriced` 策略：若配置了非 0 fallback，provider 调用前必须按请求可估算 usage 建立 fallback
     reservation / API-key hold；provider 返回后 `cost = usage_scaled_fallback_cost 或 0.0`，
     照常走 settle / `record_spend` / `settle_api_key_budget_reservation`，usage/spend 记录附带
     `unpriced=true` 标记，且不能让请求在无任何 per-key hold 的情况下先花费上游成本；
   - `reject` 策略下若预检后仍在 settle 才发现不可定价（例如 pricing 热更新、流式 usage
     形状与预估不一致），不能 drop reservation。已有 reservation 时按已预留金额结算并打
     `unpriced=true`；没有 reservation 时按 usage-scaled fallback（若配置）或 0.0 记录；
   - 任何策略下不再出现「有 usage 但预留被退回」。
6. **usage/spend 存储形状**：新增一个显式 usage 写入结构或等价 API，例如
   `UsageRecord { requests, tokens, cost, unpriced, pricing_policy, provider, model }`，替代
   只传 `(tokens, cost)` 的调用路径。持久层和 in-memory 层至少在 `KeyUsageStats` 读模型暴露
   `unpriced_requests`、`unpriced_tokens`、`unpriced_cost`、`last_unpriced_at`；如果存在明细 spend
   record/table，则每条记录也要有 `unpriced: bool` 与 `pricing_policy` 字段。日志不能替代该读模型。
7. **可观测性**：新增事件计数 metric
   `gateway_unpriced_events_total{provider,model_bucket,policy,outcome}` 和金额 metric
   `gateway_unpriced_spend_total{provider,model_bucket,policy,outcome}`。preflight reject 与
   routing candidate exclusion 只增加事件计数；settlement fallback 才增加 spend total，不能用 `1`
   污染金额 metric，也不能用 `0` 让拒绝路径不可见。`provider` 必须来自配置或
   registry 名称；`model_bucket` 只能使用配置内有界 deployment id / catalog model bucket /
   `unknown`，不能直接使用任意请求 model 字符串；原始 model 可写入结构化 error 日志。
   preflight reject、routing candidate exclusion、settlement fallback 都要增加 metric。
8. 同模式路径（`spend.rs`、`spend/pricing.rs`、`gemini/spend.rs`、`audio/budgeting.rs` 等）
   共用同一个 policy/结算辅助函数或等价 traits，避免再次漂移。

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P1 默认拒绝 | usage-aware preflight + routing candidate filter | 单测：unpriced usage + reject → 4xx，预算余额不变，provider 未调用 |
| P2 放行但结算 | spend.rs Err 分支 + UsageRecord | 单测：allow_unpriced → settle 被调用、退款不发生、usage 读模型带 unpriced 聚合 |
| P3 有 usage 必结算 | reservation 生命周期 | 单测：usage 存在时 reservation.settle 或等价 key reservation settle 必被调用 |
| P4 可观测 | metrics + structured error log | 单测/集成：reject/candidate-excluded 增加 events，settlement fallback 增加 spend total，label 有界 |
| P5 已定价不变 | 全路径 | 现有 spend 测试回归 |
| P6 priced deployment 优先 | router candidate selection | 单测：同 model 两个 deployment 时跳过 unpriced 候选并选择 priced 候选 |
| P7 cache 不绕过 policy | chat / embeddings response cache | 单测：cached hit 在 reject 策略下仍返回 `model_not_priced` |

## 数据流

请求 → 根据 endpoint 构造/估算 `PricingUsage` → cache 命中前 policy gate → router 过滤不可定价候选 → 预算预留
（usage-aware pricing gate）→ provider 调用 → provider usage → `record_settled_spend`
→（定价成功：现状不变 | 定价失败：按 policy 结算并打标）→ per-key `UsageRecord` 写入与
`KeyUsageStats` unpriced 聚合更新。

## 备选方案

- 仅升级日志为 error 并保留免费放行：不满足 invariant 1/3，预算控制仍然失效，拒绝。
- 在 pricing service 内部兜底返回 0 成本 `Ok`：把问题藏得更深，上层无法区分，拒绝。
- 全局硬编码 fail-closed 无配置：对私有部署（自建模型无定价）不友好，保留配置开关。
- 把 `unpriced_fallback_cost_per_1k_tokens` 改成固定每请求金额：命名会误导运营定价，拒绝。

## 风险

- Security: 无新增面；收紧了绕过路径。
- Compatibility: 默认行为收紧，可能影响依赖未定价模型的现有部署——CHANGELOG + 配置逃生门。
- Performance: 预留前多一次 catalog 查找（内存哈希查找，可忽略）。
- Observability: Prometheus label 必须有界，否则 typo model 会制造高基数时间序列。
- Maintenance: 结算辅助函数统一两处路径，降低漂移风险。

## 测试计划

- [ ] Unit tests: spend.rs 结算分支（reject/allow、有无 usage-scaled fallback、有无 reservation）。
- [ ] Unit tests: usage-aware pricing gate、candidate filter、4xx 错误形状。
- [ ] Unit tests: `UsageRecord` / `KeyUsageStats` unpriced 聚合读写。
- [ ] Integration tests: 带预算 key 的端到端 unpriced 请求（默认拒绝、开关放行）。
- [ ] Manual verification: `/metrics` 观察 `gateway_unpriced_spend_total`。

## 回滚方案

配置 `pricing.allow_degraded=true`（仅覆盖 pricing source 初始加载）加
`unpriced_model_policy: allow_unpriced` 且不设 fallback 价，即可恢复接近旧行为（差异：spend
记录带标记且日志/metric 保留）。代码回滚为单 PR revert。
