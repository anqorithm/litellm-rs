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

本修订在 `main@12faaf56` 复核 remaining-six，并以维护者 2026-07-15 的
[#837 决策评论](https://github.com/majiayu000/litellm-rs/issues/837#issuecomment-4982855968)
为权威输入。评论包含 `ready_to_implement` 并随后重申 ordering gates；原 T5/T6 umbrella
缺少逐 provider 的先决条件与独立验证，因此追加 remaining-six tasks，不改写 T1–T9 历史含义。
该评论只批准 remaining six，不等于批准历史 66 行矩阵中的全部 delete/non-LLM lane，原 T2/T3 保持开放。

## 目标

- 每个 native provider 目录要么有真实构造/dispatch 路径，要么被删除、降级为 catalog-only 后移除 native 模块，
  或进入带理由的显式豁免；catalog 条目不能在重复 native 模块仍存在时算作 native 可达。
- 建立防回归守护：不可达的 literal 或 macro-generated `LLMProvider` 无法再无声进入主干。
- 处置决策逐目录显式记录，维护者审批后才动手删除（U-05：不确认不删除）。
- `amazon_nova`、`github`、`meta_llama`、`v0` 各自严格执行 catalog policy/equivalence → demotion；
  `ollama` 独立 native wiring；`custom_api` 严格执行 0.6.0 deprecation → version workflow gate → 0.7.0 removal。

## 非目标

- 不改变 Provider dispatch 架构（enum vs trait object 归 #519 A-4）。
- 不新增任何 provider 功能或模型目录更新。
- 非 LLM 能力目录按实际 `ProviderCapability` / route behavior 判定；只有未声明 chat/LLM 能力者
  才能进入 non-LLM lane。search/vector/image/video/translation/embedding-only 等产品定位决策单独列 lane，
  不默认按「LLM provider 死代码」处理。

## Behavior Invariants

1. **B-001** 处置清单批准前，不删除任何目录（分类清单本身是第一个交付物）。
2. **B-002** 「wire」处置的 native 目录：接线后存在至少一条端到端可构造路径（enum/factory/dispatch 或等价构造点），
   并有 conformance 测试覆盖；docs、tests、raw text hit 不算可达性证据。
3. **B-003** 「delete」处置的目录：删除后 `cargo check --all-features` 与全量测试通过，无 dangling `pub mod`。
4. **B-004** 「demote」处置的目录：只有在 catalog runtime 能保持 endpoint 构造、auth env fallback、provider-specific
   endpoint、capability set 与现有 native 行为等价时，才可转为 `registry/catalog.rs` 条目；
   native 目录必须随后删除或进入带 issue/期限的豁免，不能让 catalog backing 掩盖重复 native 实现。
5. **B-005** 守护测试常驻：枚举 `src/core/providers/*/` 中的 literal `impl LLMProvider` 与
   `define_http_provider_with_hooks!` 等 macro-generated provider，断言均可达或在显式豁免清单
   （带 issue 引用与期限）中。
6. **B-006** 每个删除 PR 遵守仓库 PR 限制（≤10 文件 / ≤500 行规则对删除类 PR 按 tranche 拆分执行，Cargo.lock/纯删除行不计）。
7. **B-007** 删除任何 `src/core/providers/mod.rs` 导出的 `pub mod` 前，必须记录 public API / semver 影响：
   若下游 crate 可直接 import/instantiate，该 tranche 需要 breaking-change 说明或先走 deprecation lane。
8. **B-008** `custom_api` 不属于 shared infra；必须在矩阵中作为 provider lane（wire/delete/demote/exempt）单独决策。
9. **B-009** image/video/translation/embedding-only/non-chat provider 不进入 LLM delete lane，除非 non-LLM 产品决策明确批准；
   任何声明 `ProviderCapability::ChatCompletion` 的 adapter 必须走 LLM wire/delete/demote/exempt 矩阵，不能靠名称归入 non-LLM。
10. **B-010** 四个 demote provider 均先完成独立 catalog policy/equivalence 与 0.6.0 deprecation，再经 version-workflow 与 T9 compatibility gates 执行 0.7.0 demotion。
11. **B-011** 等价证据必须 provider-specific：Amazon model/pricing/capability；GitHub `GITHUB_MODELS_API_BASE` +
    model/pricing/capability/health；Meta auth/identity/filtering/streaming/model metadata/capability；V0 authoritative
    aliases/model metadata/pricing/health/error policy，禁止 no-model/zero-cost canonical fallback。
12. **B-012** `ollama` 先让普通与 streaming 请求都使用 policy-aware client，覆盖 `api_base`、
    SSRF/private-network authority 并移除 unwired raw-HTTP exception；source boundary guard 通过后，
    才使用既有 native protocol 接入 `ProviderType`/registry/factory/dispatch，且不新增 generic OpenAI catalog route。
13. **B-013** `custom_api` 先在 0.6.0 deprecate；version-bump workflow 能验证 breaking 0.7.0 bump 后才可删除。
14. **B-014** 每 task/provider 一个 PR；hard cap 为 ≤500 non-doc changed lines（按 B-006 排除纯删除行，
    non-doc additions/edits 绝不豁免），可建议 ≤4 个非纯删除文件；单一 provider directory 的纯删除
    只可例外物理 file count，且不得混入第二 provider；T7/T8 最后执行。

## 验收标准

- [ ] 历史 66 目录 baseline 的每行完整证据已补齐，且全量 delete/non-LLM/remaining lane 获维护者批复。
- [x] 守护测试合入并能在 CI 捕获「新增不可达 provider」。
- [ ] 四个 demote provider 的 policy/equivalence 与 demotion tasks 均独立完成且依赖方向正确。
- [ ] `ollama` HTTP hardening predecessor 与 native wiring 按序完成且无 generic catalog route；
      `custom_api` deprecation → workflow → T9 compatibility → removal 按序完成。
- [ ] 处置执行完成后，`src/core/providers/` 无 factory/dispatch 不可达的 native `LLMProvider`
      （含 macro-generated provider；已 demote 且 native 目录删除者除外；豁免清单除外）。
- [ ] T5/T6/T9 汇总后，T7 closure 与 T8 full verification 依次完成。
- [ ] README / CLAUDE.md 的 provider 能力描述与处置结果一致。

## 边界情况

- Empty/missing：缺 catalog model/auth/pricing/health/error data 即不满足等价，不得 fallback 后 demote。
- Duplicate/conflict：catalog 已有同名条目但 native 目录仍存在（如 v0 / meta_llama），不得据此判定 native 可达。
- Concurrency：共享 catalog/lifecycle/CHANGELOG/migration files 的任务串行；并行 agent 必须文件 ownership 不相交。
- Ordering/time：0.6.0 deprecation 先于 0.7.0 removal；T7/T8 最后。
- Permission/auth：不记录真实 token，只验证 env-key contract；workflow test 不执行 release。
- Partial failure：任一 policy、compatibility 或 workflow gate 失败即阻止后继 removal。
- Retry/idempotency：重复运行 guard/catalog/workflow checks 不改变 repo/release state。
- Migration/backward compatibility：0.6.0 保留公开 surface；0.7.0 breaking removal 提供 CHANGELOG/migration notes。
- Observability：每个 PR 记录 head SHA、命令、结果与 provider-specific evidence。
- Feature/dependency/migration：feature-gated 仍不可达、只被死代码引用或处于 `DU` 半迁移状态者，继续按原 GH837/CLAUDE.md 规则处置。
- 只被文档、测试、注释、README 或无关同名类型命中的 provider：不得据此判定为可达。

## 发布说明

0.6.0 只加入 deprecation/迁移说明；0.7.0 public removal 是显式 breaking change，须经 workflow 与 T9 gates。
`ollama` native wiring 是独立运行时能力变化；所有变更分别记录 release evidence。
