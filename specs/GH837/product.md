# Product Spec

## Linked Issue

GH-837 / #837

## 用户问题

在 `origin/main@c47596a4`，`src/core/providers/` 有 66 个目录（含 base/factory/registry/macros/thinking 等基础设施），
其中约 41 个 provider 目录携带完整实现（config/error/streaming/models/provider + tests）但从任何入口都不可构造：
`Provider` enum 只有 14 个可构造变体，`factory/registry.rs` 与 Tier-1 catalog 均不引用它们
（已抽查 `DeepgramProvider` / `OllamaProvider` 目录外零引用）。

对维护者的代价：数万行「维护中的死代码」拉长编译时间、放大 review 面、误导贡献者（以为改了就能生效）、
并使 README/文档的 provider 数量宣传与真实能力脱节。#137 曾以同样理由删除 39 个孤儿实现并标记完成，
但当前主干上孤儿目录再次达到 ~41 个——说明除了一次性清理，还需要防回归的守护测试。

## 目标

- 每个 provider 目录要么可从 factory/catalog 构造并路由，要么被删除或降级为 Tier-1 catalog 条目。
- 建立防回归守护：不可达的 `impl LLMProvider` 无法再无声进入主干。
- 处置决策逐目录显式记录，维护者审批后才动手删除（U-05：不确认不删除）。

## 非目标

- 不改变 Provider dispatch 架构（enum vs trait object 归 #519 A-4）。
- 不新增任何 provider 功能或模型目录更新。
- 非 LLM 能力目录（search/vector/工具类，如 tavily、searxng、milvus、pg_vector、firecrawl、google_pse、exa_ai）
  的产品定位决策单独列 lane，不默认按「LLM provider 死代码」处理。

## Behavior Invariants

1. 处置清单批准前，不删除任何目录（分类清单本身是第一个交付物）。
2. 「wire」处置的目录：接线后存在至少一条端到端可构造路径（factory 或 catalog），并有 conformance 测试覆盖。
3. 「delete」处置的目录：删除后 `cargo check --all-features` 与全量测试通过，无 dangling `pub mod`。
4. 「demote」处置的目录：OpenAI 兼容者转为 `registry/catalog.rs` 单行 `def()` 条目，行为经 smoke 验证等价。
5. 守护测试常驻：枚举 `src/core/providers/*/` 中的 `impl LLMProvider`，断言均可达或在显式豁免清单
   （带 issue 引用与期限）中。
6. 每个删除 PR 遵守仓库 PR 限制（≤10 文件 / ≤500 行规则对删除类 PR 按 tranche 拆分执行，Cargo.lock/纯删除行不计）。

## 验收标准

- [ ] `specs/GH837/` 附录形成 66 目录全量处置矩阵（wire / delete / demote / keep-infra / non-llm-lane）并获维护者批复。
- [ ] 守护测试合入并能在 CI 捕获「新增不可达 provider」。
- [ ] 处置执行完成后，`src/core/providers/` 无 factory 与 catalog 均不可达的 `impl LLMProvider`（豁免清单除外）。
- [ ] README / CLAUDE.md 的 provider 能力描述与处置结果一致。

## 边界情况

- 目录被 `providers-extra` / `providers-extended` feature gate 但 gate 后仍不可达：按不可达处理。
- 仅被其他死目录引用的目录（死代码引用死代码）：连带处置。
- 半迁移目录（CLAUDE.md 提到的 `DU` 状态遗留）：按 CLAUDE.md「Resolving half-migrated providers」流程归类。

## 发布说明

删除类变更不影响任何可构造路径的运行时行为；在 CHANGELOG 以 refactor 记录，并注明恢复方式（git history）。
