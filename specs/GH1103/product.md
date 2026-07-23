# Product Spec

## Linked Issue

GH-1103 / #1103

## 用户问题

#726 已让 `PricingService` 成为用户可见 pricing、cost 与 spend 计算的权威入口，
但公开 `core::cost`/`core::pricing` compatibility surface、legacy DTO、provider fallback 与新的
`pricing_service` 类型仍并存。当前没有独立 owner 定义这些兼容面的生命周期，维护者既无法安全删除，
也无法判断某个公开计算入口或 fallback 是必要兼容还是仍在维持第二套 pricing authority。

## 目标

- 为已发布 `core::cost`、`core::providers::base::pricing`、`utils::{ModelUtils, TokenUtils}` pricing method、
  feature-gated provider pricing path/re-export，以及 post-v0.5 的当前公开 `core::pricing` facade 的 symbol、
  production consumer、compatibility DTO、`PricingService` live provider fallback source 和 test-only consumer
  建立完整 inventory；public compatibility 基线只取 `v0.5.0` tag/package 在 default、no-default/lite 与
  docs.rs 对应 feature set 下实际发布的 surface。
- 保持 `PricingService` 是唯一 user-visible pricing authority，不恢复第二套 live pricing source。
- 将必须兼容的 surface 明确为 adapter；仅对维护者批准的 surface 在 0.6.x 标记 deprecated。
- 为 0.7.0 removal 定义 release、semver、public API approval 与行为回归门禁。
- 把 #519 A-6 的剩余 ownership 收敛到本 issue，同时保持 #726 已完成的 runtime authority 结论。

## 非目标

- 不刷新模型价格或 provider capability 数据。
- 不改变 provider selector、registry、router 或 runtime convergence；这些边界分别归 #837、#965 与 #519 A-4。
- 不改变 pricing endpoint、预算持久化、reservation/settlement 或 spend 语义。
- 不把 provider 内部仅用于展示、本地 catalog 或 protocol-specific 计算的类型机械纳入删除范围；但凡被
  `PricingService` authority 的 live fallback 调用链读取的 catalog/source 必须进入 inventory 与 disposition。
- 不在 spec approval、0.6 release evidence 与 version workflow gate 之前删除 public API。

## Behavior Invariants

1. 对同一已定价 provider、model、usage 与 pricing source，pricing route、budget reservation、spend settlement、
   callback lifecycle terminal cost，以及 chat/embedding response-cache admission pricing gate 必须使用同一个
   runtime `PricingService` configured/loaded authority 结果；custom `pricing.source` parity 只在这些 live consumers
   之间比较。保留 v0.5 签名、无法接收 runtime service 的 `core::cost`、provider-base pricing 与
   utility compatibility facade 只与 `PricingService::with_embedded_default()` 的同 embedded source 结果比较，
   不得声称它与 custom source 数值相等。
   unknown/incomplete pricing 在默认 `Reject` policy 下 fail closed，而显式 `AllowUnpriced` policy 必须继续以配置的
   fallback cost 保持 reservation/settlement parity。
2. 0.6.x 必须保持 `v0.5.0` tag/package 已发布 `core::cost` 与
   `core::providers::base::{pricing, PricingDatabase, get_pricing_db}`、
   `utils::ModelUtils::get_model_pricing`、`utils::TokenUtils::calculate_cost` 及对应 public re-export/module path，
   以及 v0.5 no-default/lite 和 docs.rs feature set 暴露的 pricing/cost API（docs.rs 至少 Azure/Bedrock）的签名
   与既有兼容行为；
   只有 inventory 中经维护者批准的已发布 symbol 可以标记 deprecated，且必须给出替代路径。
   `core::pricing` 是 post-v0.5 current-head surface，仍必须完成 authority disposition，但不得误列为 v0.5
   published compatibility cohort，也不因当前存在而自动套用 0.6 deprecate/0.7 remove 窗口。
3. 所有 user-visible provider fallback 的 lookup/calculation 必须在 `PricingService` authority 内执行；
   `core::cost` adapter 只能转换 canonical DTO/error；公开 `core::pricing` 中可独立加载或计算价格的 facade
   必须同时获得 compatibility lifecycle 与 authority disposition，未完成决策不得视为 `keep_adapter`。
   未归类 fallback 不得删除，也不得继续作为第二套 live pricing source；live gateway 在默认 `Reject` policy
   下不得把 unknown/incomplete pricing 静默变成 0 美元。
4. 0.7.0 removal 只能删除已在 0.6.x 发布为 deprecated、已有迁移说明、并由 public compatibility fixture
   覆盖的 surface；任何额外 public break 都需要独立批准。
5. provider alias、tiered pricing、cached/reasoning/image/audio token extras 与 time-based pricing 的既有结果
   在兼容窗口内不得改变。
6. 此生命周期工作不得改变 endpoint shape、持久化数据、权限、网络调用或 request routing。
7. #519 A-3 继续以 #729 的 canonical `LLMProvider + ProviderCapability` 决策为准；A-4 继续由 #965 承接，
   本 issue 不重新打开这些架构路线。

## 验收标准

- [ ] inventory 从 `v0.5.0@de594c81` tag/package public API 基线起步；published cohort 覆盖当时实际存在的
  `core::cost`、`core::providers::base::pricing`、`utils::{ModelUtils, TokenUtils}` 的全部 public re-export/module path
  与 pricing methods，以及 default、`--no-default-features --features lite`、docs.rs feature matrix 下的
  provider pricing/cost API；current-head cohort 另覆盖
  post-v0.5 `core::pricing` symbol/production import、legacy DTO、`pricing_service/authority.rs::provider_catalog_model_info`
  及其 Azure、Bedrock、Amazon Nova、xAI authority source、legacy fallback、callback lifecycle、
  chat/embedding response-cache pricing gate 和 test-only consumer；source guard 防止遗漏或 cohort 混淆。
- [ ] 兼容矩阵仅对 v0.5 published adapter 记录 `keep_adapter`、`deprecate_0_6_remove_0_7` 或
  `needs_decision`；post-v0.5 `core::pricing` 显式记录 `post_v0_5_unreleased` baseline status，任何
  authority-bearing public facade 与 user-visible fallback 另记录 `migrate_authority` 或 `needs_decision`，并附
  owner evidence，不能仅以 current-head public 或 `keep_adapter` 为由保留独立计算。
- [ ] 0.6.x tranche 保持相对 `v0.5.0` 的 public compatibility 与 runtime behavior，并提供 CHANGELOG、迁移说明，
  以及三组 tag/package-derived compile/behavior fixtures：(a) default features；(b)
  `--no-default-features --features lite` API-only；(c) v0.5 docs.rs exact set
  `gateway,postgres,sqlite,redis,s3,metrics,tracing,websockets,analytics,providers-extra,providers-extended`；fixture
  必须覆盖 lite lane 的 public pricing imports/behavior 与 docs.rs feature-gated Azure/Bedrock pricing/cost path，
  而非只验证默认编译。
- [ ] source-aware parity 分为两组独立回归：custom `pricing.source` 的 pricing route/budget reservation/spend
  settlement、callback terminal cost、chat/embedding response-cache admission pricing gate 共享同一 runtime loaded
  authority；embedded compatibility facade 只与 embedded authority parity。
  未经另行批准，不假设新增 authority injection 参数或更改 v0.5 签名，也不做跨 source 数值相等断言。
- [ ] `PricingService` authority、spend parity、provider alias/fallback、默认 `Reject` policy 的 unknown pricing
  fail-closed 与显式 `AllowUnpriced` policy 的 fallback reservation/settlement parity 回归全部通过。
- [ ] 0.7.0 removal 仅在 version workflow、已验证 0.6 release artifact 和 human public-API approval 均满足后执行。
- [ ] #519 roadmap 能明确链接本 issue 作为 A-6 剩余 ownership，不再暗示 A-3/A-4 无 owner。

## 边界情况

- 下游 crate 可能只 import DTO 或 trait，而不调用 gateway；compile fixture 必须覆盖这种 library-only consumer。
- provider-local 类型与共享 compatibility DTO 同名时，inventory 必须按完整 module path 区分。
- bundled pricing 初始化失败时，兼容 adapter 不得通过空 authority 或 fallback 静默低估用户可见成本。
- `core::pricing::{PricingDatabase, GLOBAL_PRICING_DB, get_pricing_db, calculate_cost}` 及其公开方法即使只被下游
  library 使用，也必须按完整 path 进入 current-head authority disposition，不能因 gateway 未直接调用而遗漏；
  但该 module 由 `04c0774a` 在 `v0.5.0@de594c81` 之后新增，不能据此生成 v0.5 published import fixture。
- `utils::ModelUtils` 与 `utils::TokenUtils` 同时经 `utils`、`utils::ai` 与更深 module path 公开；inventory 必须按
  完整 import path 记录 `get_model_pricing`/`calculate_cost` 的独立 lookup/calculation authority disposition，不能
  只盘点 struct 的顶层 re-export。
- custom `pricing.source` 可故意覆盖 embedded 价格；正确性是同 source consumer parity，不是 custom 与 embedded
  价格相等。若未来要让无 service 参数的 v0.5 facade 接收 runtime authority，必须另行批准 non-breaking injection 设计。
- callback terminal cost 或 cache-enabled deployment 的 admission gate 若偏离 route/reservation/settlement 的
  configured source，必须显式失败；不得用 embedded fallback 或 cache hit 掩盖 source divergence。
- `AllowUnpriced` 是显式 policy，不是 pricing miss 的隐式零成本降级；其配置 fallback cost、reservation、
  settlement 与 usage record 必须维持同一语义。
- 在 0.6.x 新增的合法 adapter 不自动进入 0.7 removal；必须先补齐矩阵与批准证据。
- 0.7.0 之前若 version workflow 仍不能正确处理 0.x breaking bump，removal 必须保持 blocked。

## 发布说明

仓库当前版本为 0.5.0；`implx auto` 的默认 deprecation window 采用下一个 minor 0.6.0。
0.6.x 只允许保留行为的 targeted deprecation，0.7.0 才允许经批准的 breaking removal。
CHANGELOG 与迁移文档必须列出完整 Rust import 替代路径，不能把 runtime 行为未变化误写成功能删除。
