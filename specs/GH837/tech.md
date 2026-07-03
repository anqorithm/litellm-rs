# Tech Spec

## Linked Issue

GH-837 / #837

## Product Spec

Link to `product.md`.

## Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| Provider enum | `src/core/providers/mod.rs:406-430` | 14 个可构造变体（3 个 `providers-extra`、6 个 `providers-extended` gate） | 可达性的第一入口 |
| Factory | `src/core/providers/factory/registry.rs:52-258` | `from_config_async` 分支即全部 Tier-2 构造点 | 可达性判定依据 |
| Tier-1 catalog | `src/core/providers/registry/catalog.rs` | 数据驱动 OpenAI 兼容 provider → `Provider::OpenAILike`；catalog path 不等于同名 native module 可达 | demote 目标位置，但不能掩盖重复 native 实现 |
| Public exports | `src/core/providers/mod.rs` | 多个孤儿目录仍以 `pub mod` 导出，feature gate 后下游 crate 可能直接 import | 删除前必须评估 public API / semver |
| Macro providers | `src/core/providers/*/provider.rs` + `src/core/providers/macros/` | `custom_api`、`deepl` 等通过 `define_http_provider_with_hooks!` 生成 `LLMProvider` impl | 守护测试不能只搜 literal `impl LLMProvider` |
| Non-LLM surfaces | `runwayml`、`recraft`、`stability`、`deepl`、search/vector/embedding-only 目录 | 可能暴露 image/video/translation/search/vector/embedding capability，而非 chat LLM | 进入 non-llm-lane，不走 LLM delete lane |
| Orphan dirs | `src/core/providers/{custom_api,deepgram,ollama,elevenlabs,huggingface,sagemaker,watsonx,voyage,databricks,triton,jina,...}` | `pub mod` 声明 + 完整实现，但无 native factory/dispatch 构造点 | 处置对象（~41 个） |
| Registry types | `src/core/providers/registry/{types.rs,lifecycle.rs,support_matrix.rs}` | `PROVIDER_TYPE_REGISTRY` 等元数据 | 守护测试挂载点 |
| Prior art | #137 / #140 / #714（均 CLOSED） | 清理过一轮后回归 | 说明需要守护测试 |

## 设计方案

**Phase 1 — 处置矩阵（本 spec 附录，人工批复）**

对 66 个目录逐一标注六类 lane：

- `keep-infra`：base、factory、registry、macros、thinking、openai_like；仅限 shared infra。
- `wired-native`：openai、anthropic、bedrock、mistral、cloudflare、azure、azure_ai、vertex_ai、gemini、
  github_copilot、fal_ai、cohere、replicate 等已有 native enum/factory/dispatch 构造点者。
- `catalog-only-with-native-duplicate`：catalog 已支持但同名 native 目录仍存在者（当前至少 v0 / meta_llama）；
  catalog 条目不能算 native module 可达，必须转为 `demote-to-catalog` 删除 native 目录，或进入显式豁免。
- `demote-to-catalog`：OpenAI 兼容且无自定义流式/鉴权者（候选：baseten、codestral、empower、
  datarobot、gradient_ai、morph、predibase、vercel_ai 等，逐个验证 API 形状）；
  Snowflake 这类自定义 endpoint/auth 的 provider 不得放入 catalog demote 候选，必须 wire/delete/exempt
  或保留 native custom handling。demote 完成条件必须包含 native 目录删除或显式豁免。
- `delete-native`：chat/LLM native module 非 OpenAI 兼容、无用户需求证据、无构造点，且 public API 影响已记录者
  （候选需从矩阵证据得出；不得把 image/video/translation/search/vector/embedding-only provider 混入）。
- `non-llm-lane`：只能由 declared capability / route behavior 推导，不能按名称 seed。若 provider
  声明 `ProviderCapability::ChatCompletion`（例如某些 search/translation adapters），必须回到
  LLM wire/delete/demote/exempt 矩阵；只有纯 search/vector/audio/image/video/embedding-only 等
  非 chat 能力才先决定产品上是否保留，再决定 wire/delete。
- `exempt`：如 `custom_api` 这类不是 shared infra、但需要产品/架构单独决策的 provider；必须记录 issue、
  owner、期限和后续 lane，不能永久静默豁免。

判定脚本（附录附命令）：不得使用裸 `rg "<TypeName>" src` 作为可达性证据。每目录至少记录：

- native construction/dispatch evidence：`Provider` enum variant、`ProviderType` match arm、factory `Box::new(...)` /
  `Arc::new(...)`、route selector 中的 typed dispatch，或等价 Rust symbol；
- catalog evidence：`registry/catalog.rs` 的 `def()` 只能证明 `Provider::OpenAILike` 路径存在；
  当同名 native 目录仍存在时，不从 native orphan set 中扣除；
- public export evidence：`src/core/providers/mod.rs` 中 `pub mod <dir>` 与 feature gate；
- provider implementation evidence：literal `impl LLMProvider`、`define_http_provider_with_hooks!`、
  `define_pooled_http_provider_with_hooks!` 等 macro invocation；
- capability evidence：`ProviderCapability::*` 或 model metadata，用于把 image/video/translation/search/vector/embedding-only
  provider 放入 non-llm-lane。
- internal dependency / metadata-use evidence：非 dispatch 运行时代码对 provider 目录内部类型的依赖
  （例如 pricing/cost metadata registry）必须单独记录；有内部依赖的目录不能仅凭无 factory route
  直接 delete/demote，需先迁移依赖或拆出 shared metadata。

文档、README、注释、tests、无关同名 struct（如 A2A 的 LangGraph 类型）只可作为参考，不可作为可达性判定。

**Phase 2 — 守护测试（先行合入）**

在 `registry` 增加 conformance 测试：扫描 `src/core/providers/*/` 的 literal `impl LLMProvider` 类型名、
`define_http_provider_with_hooks!` 与 `define_pooled_http_provider_with_hooks!` 等 macro-generated provider 名称，
与「native enum/factory/dispatch 构造点 + catalog-only 完成状态 + 维护者批复的临时 orphan baseline +
豁免清单」求差集。新增或未批准 orphan 非空即失败；已在 #837 批复矩阵中排入 delete/demote/non-LLM lane
的当前 orphan 可作为临时 baseline，带 issue、owner、期限和退出条件。关键规则：

- catalog 条目只在 native 目录不存在、或该目录被显式豁免时，才可满足该 provider 的最终可达状态；
- `custom_api`、pooled-hook provider（如 `ai21`、`amazon_nova`、`datarobot`、`empower`、`firecrawl`）
  等 macro provider 必须出现在扫描结果中，不能因无 literal impl 被漏掉；
- non-LLM provider 进入独立 lane，不得被 LLM delete guard 自动要求删除；
- 豁免清单与临时 baseline 为带 issue 引用、owner、期限和退出条件的常量表，CI 可见；T5/T6
  每删除或 demote 一个目录必须同步收缩 baseline，最终收尾时 baseline 为空。

**Phase 3 — 分批执行**

- delete lane：按目录家族分 tranche（每 PR 一个或数个小目录），纯删除 + `pub mod` 清理；每个 tranche
  先记录 public API/semver 影响，必要时使用 breaking-change commit 或 deprecation 过渡。
- demote lane：每 PR 一个 provider：确认已有 catalog route 或取得维护者对新增 catalog route 的产品批准 →
  加 catalog `def()` → 删 native 目录 → smoke 验证模型列表/鉴权头等价；若 native 目录暂留，必须在豁免清单登记，
  不能把 catalog 当 native 可达。新增 catalog-backed selector 属于 runtime behavior change，不能作为纯清理默认发生。
- wire lane（如维护者选择保留个别）：按 CLAUDE.md Tier-2 流程补 enum/factory/dispatch。

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P2 wire 可达 | factory/enum | conformance 测试 + 单测构造 |
| P3 delete 干净 | providers/mod.rs | `cargo check --all-features` + `rg` 无 dangling mod |
| P4 demote 等价 | catalog.rs + native dir removal | catalog smoke 测试（base_url/env key/名称）+ 无重复 native impl |
| P5 守护常驻 | registry conformance test | CI 上人为引入 literal impl 与 macro provider 孤儿目录的负测试 |
| P7 public API | providers/mod.rs / CHANGELOG | 删除导出模块前有 semver/compatibility 记录 |
| P9 non-LLM 范围 | capability scan / matrix | image/video/translation/embedding-only provider 未进入 LLM delete lane |

## 数据流

无运行时数据流变化；delete/demote 仅移除不可达代码路径。

## 备选方案

- 全部接线（wire all）：~41 个目录补 enum/factory/dispatch，扩大 #519 反对的封闭 enum 面，且多数无用户需求证据，拒绝。
- 保持现状仅加守护：阻止恶化但不解决存量数万行死代码，拒绝。
- 移入独立 `providers-graveyard` feature：仍参与编译与 review 面，拒绝。

## 风险

- Security: 无；删除减少攻击面。
- Compatibility: gateway routing 不受不可达 native module 删除影响；但 `pub mod` 导出的 provider 可能被下游 crate
  直接 import/instantiate，删除属于潜在 public API break，必须逐 tranche 记录 semver/compatibility 决策。
- Performance: 编译时间预期显著下降（删除数万行 + 各目录 tests）。
- Maintenance: 主要风险是误删 keep-infra 依赖，靠 `cargo check --all-features` 与全量测试兜底。

## 测试计划

- [ ] Unit tests: conformance 守护测试（含豁免清单机制、macro-generated provider fixture、
      catalog/native duplicate fixture）。
- [ ] Integration tests: demote 后 catalog smoke 测试。
- [ ] Manual verification: 处置矩阵逐行与 construction/dispatch/public-export/capability 证据核对；
      raw text hits 不作为通过条件。

## 回滚方案

删除类 PR 逐个 revert 即可恢复；git history 保留全部实现。守护测试可通过豁免清单临时放行。
