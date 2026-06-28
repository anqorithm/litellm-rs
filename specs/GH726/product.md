# Product Spec

## Linked Issue

GH-726 / #726

## User Problem

网关的价格、成本和 spend 记录现在走多条路径。`/v1/pricing/*`
接口使用 `PricingService`，但 AI 请求预算预留和成功后的 spend 落账仍直接使用
`core::cost` 的旧计算器和旧 DTO。这样会让同一个模型在价格 API、预算预留、
预算结算和 provider-specific 计费中出现不一致，也会让缺失价格在用户可见
成本场景中被错误地当作 0 美元处理。

## Goals

- 让 `PricingService` 成为用户可见 pricing、cost、spend 计算的权威入口。
- 保留需要兼容的 `core::cost` API，但把它收敛为薄适配层，而不是第二套价格来源。
- 让预算预留和完成后 spend 结算使用同一套 provider/model 价格匹配规则。
- 对缺失或不完整价格保持 fail-closed，避免用户可见成本被静默低估。
- 用测试覆盖 pricing route、AI spend/reservation 路径和至少一个 provider-specific
  pricing 场景。

## Non-Goals

- 不刷新模型价格数据，除非测试夹具需要最小本地数据。
- 不改变 provider registry 或 provider selector 语义。
- 不拆分 U-16 超大 Rust 文件；该维护工作属于 GH-727。
- 不移除所有 provider 内部的展示型或本地目录价格结构，除非它们已经参与用户可见 spend。

## Behavior Invariants

1. `/v1/pricing/calculate` 和 AI 请求 spend/reservation 对同一 provider/model/usage
   使用同一个权威价格计算路径。
2. 预算预留的估算成本和成功完成后的实际成本不能来自不同价格来源。
3. 用户可见 spend 场景中，未知模型、未知 provider 或 chat/completion 缺一侧 token
   价格时必须返回错误或跳过 budget spend 并记录错误，不能静默按 0 美元计费。
4. 兼容 API 如 `generic_cost_per_token`、`estimate_cost` 和 `get_model_pricing`
   可以继续存在，但必须通过 `PricingService` 的 authority 逻辑取得价格和成本结果。
5. provider-specific 价格匹配行为必须保留，包括 Xiaomi MiMo 通过 Anthropic-compatible
   路径计费、Gemini/Vertex AI alias、OpenAI-like provider-prefixed model 等既有场景。

## Acceptance Criteria

- [ ] pricing API cost calculation and AI spend/reservation use the same pricing authority.
- [ ] Legacy `core::cost` calculation functions are thin adapters over `PricingService` authority logic.
- [ ] Unknown or incomplete pricing remains fail-closed for user-visible spend.
- [ ] Tests cover pricing route behavior, AI spend/reservation behavior, and one provider-specific pricing case.
- [ ] SpecRail workflow check, focused pricing/spend tests, formatting, and all-features compile pass locally.

## Edge Cases

- Cached, reasoning, image, and audio token extras must keep their existing cost behavior.
- Time-based pricing must still require duration input where applicable.
- Provider aliases must not accidentally match unrelated provider rows.
- Pricing refresh network failure is outside this issue; this slice should rely on the already loaded
  pricing data source or bundled defaults.
- Missing usage from an upstream provider should keep the existing reservation settlement behavior.

## Rollout Notes

This is an internal architecture convergence slice. No data migration or public endpoint shape change
is expected. The main compatibility risk is callers that import `core::cost`; they should keep compiling
while receiving results from the same PricingService-backed calculation path.
