# Product Spec

## Linked Issue

GH-1113 / #1113

## 用户问题

Vertex AI 与 Gemini 当前存在多条可独立计算 token cost 的公开路径。Vertex provider 通过
model substring 选择硬编码价格并把 unknown model 记为 `0.0`；Vertex Gemini helper 对
unknown model 默认套用 Flash 价格；Gemini helper 还以 `Option<f64>` 表达失败。与此同时，
预算预留和 spend settlement 已使用 `PricingService`。同一 provider/model/usage 因入口不同
可能得到不同价格、错误模型的价格或静默零成本，破坏预算、spend 与审计可信度。

## 目标

- 在 GH1112 的 exact canonical Google model ID 与 Developer/Vertex availability overlay 之上，
  让 `PricingService` 成为 Gemini/Vertex user-visible token pricing 的唯一 authority。
- 删除或使 duplicate substring、default-Flash、unknown-zero 计算路径不可达。
- 将 Gemini/Vertex public cost helpers 收敛为 typed `Result` adapter；unknown、missing 或
  incomplete pricing 必须保留 provider/model 上下文并显式失败。
- 单一化 per-token、per-1k 与 per-million 单位转换，并证明 public helper、budget reservation
  与 spend settlement 对同一 resolved provider/model/usage 得到相同结果。
- `AllowUnpriced` 只允许由显式 request-time policy 绕过 typed pricing failure，并保留现有
  structured unpriced event/spend evidence 与可用时的 `UsageRecord::unpriced` 审计记录。

## 非目标

- 不新增、刷新或猜测 Google 模型价格；价格数据刷新归 GH1108。
- 不重新设计 GH1112 的 catalog、availability、alias、request contract、Developer API key
  或 Vertex Bearer/endpoint 边界。
- 不改变 provider selection、fallback order、retry policy、endpoint shape、持久化 schema
  或 budget limit 算法。
- 不处理 Vertex image-generation、text-to-speech 等非 Gemini token pricing helper。
- 不把 `AllowUnpriced` 变成 provider helper 的默认行为，也不让 public helper 接收或推断
  gateway policy。

## Behavior Invariants

1. **B-001** Gemini Developer 与 Vertex Gemini 的 user-visible token cost 必须先按 provider
   surface 通过 GH1112 catalog 做 exact canonical-ID 与 availability 校验，再以
   `(pricing_provider, canonical_model_id)` 查询同一个 runtime `PricingService`；不得使用
   lowercase/substring/family 猜测、默认 model 或跨 surface availability 代替 exact lookup。
2. **B-002** unknown、retired、surface-unavailable、missing-price、只有单边 token price、
   negative、NaN 或其他 incomplete pricing 必须返回 typed error；不得返回成功 `0.0`、默认
   Flash/Pro 价格、空 pricing record 或隐式 embedded fallback。
3. **B-003** provider trait/facade 与所有在本 scope 内发布的 Gemini/Vertex token-cost helper
   均返回 typed `Result<f64, ...>`，并薄适配 canonical authority。现有
   `LLMProvider::calculate_cost`/`Provider::calculate_cost` 保持
   `Result<f64, ProviderError>`；Gemini inherent/basic/multimodal helper 与 Vertex Gemini helper
   不再公开 `Option<f64>` 或裸 `f64` 失败语义。不得同时保留可调用的 legacy duplicate calculator。
4. **B-004** authority 的内部价格单位固定为 USD/token。来源为 per-1k 时只转换一次
   `per_token = per_1k / 1_000`；来源为 per-million 时只转换一次
   `per_token = per_million / 1_000_000`；总价仅为各 usage unit 与对应 per-token rate 的
   乘积之和。任何 adapter 不得再次缩放。
5. **B-005** 同一 provider、canonical model、usage 和 pricing source 经 public helper、
   budget reservation、settlement、spend record 与 callback terminal cost 必须使用同一
   resolved identity 和 canonical cost；zero-token usage 可成功为 `0.0`，但只有在 model 和
   必需 price fields 均合法时，不能把 zero usage 当作跳过 pricing validation。
6. **B-006** pricing preflight failure 在默认 `Reject` policy 下必须在 upstream、reservation、
   successful callback/cache outcome 和 spend write 前失败。retry/fallback 只能对实际选择的
   新 deployment 重新执行 exact lookup，不能借用前一个或相似 model 的价格。
7. **B-007** `AllowUnpriced` 不是 provider helper 或 authority 的返回分支。只有 gateway
   request-time code 观察到 typed model-not-priced failure 且最终合并配置明确选择
   `AllowUnpriced` 时，才可按配置的 fallback cost 继续；其他错误类型不得被该 policy 吞掉。
8. **B-008** 每次 `AllowUnpriced` 绕过必须携带原始 provider、model、policy、outcome 与
   fallback cost，记录现有 unpriced event/spend metric 和结构化 error log；存在 API-key
   context 时还必须写入 `UsageRecord::unpriced`。reservation、settlement、usage record
   的 fallback cost 必须一致；记录失败必须显式 `error`，不得伪装为 priced success。
9. **B-009** public helper 的 typed error 必须稳定区分 model not found/unavailable 与
   invalid/incomplete pricing，并保留 provider + canonical/requested model 上下文；error、
   Debug、Display、metric label 与 audit record 不包含 Gemini API key、Vertex Bearer token、
   project、location、prompt 或 response 内容。
10. **B-010** GH1112 的 catalog/availability/auth/request-contract 行为保持不变；本 issue
    只能读取其 crate-private exact-ID API。partner models 继续走现有独立 pricing owner，
    不得因 Gemini 收敛被错误路由到 Google pricing。

## 验收标准

- [ ] Fresh implementation base 已包含 GH1112 合并后的 exact Google catalog API；spec manifest
  按该 exact head 重锚，未合并前 implementation route 保持 blocked。
- [ ] Gemini Developer 与 Vertex Gemini 的同一 canonical model/usage 只调用一个
  `PricingService` authority；source/call-graph guard 证明 substring/default-Flash/unknown-zero
  calculator 已删除或不可达，且无第二份价格表参与 user-visible cost。
- [ ] `LLMProvider`、`Provider` facade、Gemini inherent/basic/multimodal helper 与 Vertex Gemini
  helper 的 scope inventory 全部有 typed `Result` disposition；公开失败不再使用 `Option`/裸
  `f64`，迁移说明列出旧/新签名及 error mapping。
- [ ] 1 token、1k tokens、1M tokens、mixed input/output、zero usage 与大数边界 fixture
  证明单位只转换一次，结果在 helper/reservation/settlement/spend/callback 间一致。
- [ ] unknown、retired、wrong-surface、missing/incomplete/invalid price 对 public helper 与默认
  gateway path 都 typed fail closed，且 upstream/network、budget reservation、priced spend、
  successful callback/cache side-effect counters 为零。
- [ ] 显式 `AllowUnpriced` 正负矩阵证明：只有 model-not-priced failure 可进入 policy；
  fallback reservation/settlement/usage-record cost 一致；metric、structured log 与
  `UsageRecord::unpriced`（有 key 时）保留 provider/model/policy/outcome 证据；默认 `Reject`
  与非 pricing error 不得绕过。
- [ ] Gemini API key 与 Vertex Bearer 仍按 GH1112 隔离，pricing/error/audit evidence 不含 secret
  或请求内容；partner-model pricing regression 不变。
- [ ] Focused tests、coverage、fmt/check/strict Clippy/full test、SpecRail gates、independent
  exact-head review、GitHub CI、review threads 和 PR gate 全部 fresh/green。

## 边界情况

- requested model 带显式 provider prefix 时，只能由 GH1112 批准的 exact canonicalization
  去除/验证 prefix；未知 prefix、空 ID 或额外 suffix 不得模糊命中。
- 同一 canonical ID 在 Developer 与 Vertex availability 不同时，pricing lookup 必须先按
  surface 拒绝 unavailable 入口，即使 pricing source 中存在同名记录。
- zero input + zero output 仍先验证 provider/model/price completeness，再返回合法 `0.0`。
- 只有 input 或只有 output tokens 时，仍要求该 usage 实际需要的字段存在；不得用缺失字段
  的 `0.0` 代替 typed error。
- custom `pricing.source` 可以与 embedded source 数值不同；live gateway consumers 对
  configured runtime source 做 parity，无法接收 runtime service 的 compatibility helper 只能
  显式使用 embedded authority，且不得被 live gateway 调用。
- `AllowUnpriced` fallback cost 可为显式配置的零，但仍必须标为 unpriced 并产生完整审计
  evidence；零 fallback 不得变成普通 priced success。
- audit/usage-record 写入失败不得回滚已经发生的 provider response，但必须显式 error，并在
  closure evidence 中可观察；不得 silent swallow。

## 发布说明

本改动会把 scope 内仍返回 `Option<f64>`/裸 `f64` 的 public cost helper 改为 typed
`Result`。实现 PR 必须提供 Rust migration note；unknown model 从静默零价/默认 Flash
变为显式错误是有意的 correctness change。`AllowUnpriced` 的 gateway 配置入口保留，但
只作为明确、可审计的 request-time policy。
