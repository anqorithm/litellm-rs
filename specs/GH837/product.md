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

- 每个 native provider 目录要么有真实构造/dispatch 路径，要么被删除、降级为 catalog-only 后移除 native 模块，
  或进入带理由的显式豁免；catalog 条目不能在重复 native 模块仍存在时算作 native 可达。
- 建立防回归守护：不可达的 literal 或 macro-generated `LLMProvider` 无法再无声进入主干。
- 处置决策逐目录显式记录，维护者审批后才动手删除（U-05：不确认不删除）。

## 非目标

- 不改变 Provider dispatch 架构（enum vs trait object 归 #519 A-4）。
- 不新增任何 provider 功能或模型目录更新。
- 非 LLM 能力目录按实际 `ProviderCapability` / route behavior 判定；只有未声明 chat/LLM 能力者
  才能进入 non-LLM lane。search/vector/image/video/translation/embedding-only 等产品定位决策单独列 lane，
  不默认按「LLM provider 死代码」处理。

## Behavior Invariants

1. 处置清单批准前，不删除任何目录（分类清单本身是第一个交付物）。
2. 「wire」处置的 native 目录：接线后存在至少一条端到端可构造路径（enum/factory/dispatch 或等价构造点），
   并有 conformance 测试覆盖；docs、tests、raw text hit 不算可达性证据。
3. 「delete」处置的目录：删除后 `cargo check --all-features` 与全量测试通过，无 dangling `pub mod`。
4. 「demote」处置的目录：只有在 catalog runtime 能保持 endpoint 构造、auth env fallback、provider-specific
   endpoint、capability set 与现有 native 行为等价时，才可转为 `registry/catalog.rs` 条目；
   native 目录必须随后删除或进入带 issue/期限的豁免，不能让 catalog backing 掩盖重复 native 实现。
5. 守护测试常驻：枚举 `src/core/providers/*/` 中的 literal `impl LLMProvider` 与
   `define_http_provider_with_hooks!` 等 macro-generated provider，断言均可达或在显式豁免清单
   （带 issue 引用与期限）中。
6. 每个删除 PR 遵守仓库 PR 限制（≤10 文件 / ≤500 行规则对删除类 PR 按 tranche 拆分执行，Cargo.lock/纯删除行不计）。
7. 删除任何 `src/core/providers/mod.rs` 导出的 `pub mod` 前，必须记录 public API / semver 影响：
   若下游 crate 可直接 import/instantiate，该 tranche 需要 breaking-change 说明或先走 deprecation lane。
8. `custom_api` 不属于 shared infra；必须在矩阵中作为 provider lane（wire/delete/demote/exempt）单独决策。
9. image/video/translation/embedding-only/non-chat provider 不进入 LLM delete lane，除非 non-LLM 产品决策明确批准；
   任何声明 `ProviderCapability::ChatCompletion` 的 adapter 必须走 LLM wire/delete/demote/exempt 矩阵，不能靠名称归入 non-LLM。

## 验收标准

- [ ] `specs/GH837/` 附录形成 66 目录全量处置矩阵（wired-native / delete-native / demote-to-catalog /
      keep-infra / non-llm-lane / exempt），并获维护者批复。
- [ ] 守护测试合入并能在 CI 捕获「新增不可达 provider」。
- [ ] 处置执行完成后，`src/core/providers/` 无 factory/dispatch 不可达的 native `LLMProvider`
      （含 macro-generated provider；已 demote 且 native 目录删除者除外；豁免清单除外）。
- [ ] README / CLAUDE.md 的 provider 能力描述与处置结果一致。

## 边界情况

- 目录被 `providers-extra` / `providers-extended` feature gate 但 gate 后仍不可达：按不可达处理。
- 仅被其他死目录引用的目录（死代码引用死代码）：连带处置。
- 半迁移目录（CLAUDE.md 提到的 `DU` 状态遗留）：按 CLAUDE.md「Resolving half-migrated providers」流程归类。
- catalog 已有同名条目但 native 目录仍存在（如 v0 / meta_llama）：catalog 只能证明 catalog path 可用，
  不能证明 native module 可达。
- 只被文档、测试、注释、README 或无关同名类型命中的 provider：不得据此判定为可达。

## 发布说明

删除类变更不影响 gateway 可构造路径的运行时行为；若删除了对下游 crate 可见的 `pub mod` 或类型导出，
按 public API 变更记录 semver/compatibility 说明，否则在 CHANGELOG 以 refactor 记录，并注明恢复方式（git history）。
