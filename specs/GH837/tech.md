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
| Non-LLM surfaces | `runwayml`、`recraft`、`stability`、audio/vector/embedding-only 目录 | 可能暴露 image/video/audio/vector/embedding capability，而非 chat LLM；声明 `ProviderCapability::ChatCompletion` 的 translation/search adapter 必须回到 LLM lane | 进入 non-llm-lane 前必须用 capability 证据排除 chat surface |
| Historical orphan baseline | `src/core/providers/{custom_api,deepgram,ollama,elevenlabs,huggingface,sagemaker,watsonx,voyage,databricks,triton,jina,...}` | 原 baseline 中为 `pub mod` + 完整实现但无 native factory/dispatch；部分目录后来已删除 | 历史审计记录，不代表当前目录计数 |
| Registry types | `src/core/providers/registry/{types.rs,lifecycle.rs,support_matrix.rs}` | `PROVIDER_TYPE_REGISTRY` 等元数据 | 守护测试挂载点 |
| Prior art | #137 / #140 / #714（均 CLOSED） | 清理过一轮后回归 | 说明需要守护测试 |

## Maintainer decision and remaining-six gap

权威输入是维护者 2026-07-15 的 [#837 评论](https://github.com/majiayu000/litellm-rs/issues/837#issuecomment-4982855968)：

- `amazon_nova`: catalog model/pricing/capability equivalence → demote。
- `github`: 保留 `GITHUB_MODELS_API_BASE` 与 model/pricing/capability/health → demote。
- `meta_llama`: auth/identity/filtering/streaming/model metadata/capability equivalence → demote。
- `v0`: authoritative aliases/model metadata/pricing/health/error policy（禁止 empty/zero-cost canonical fallback）→ demote。
- `ollama`: existing native protocol → core `ProviderType`/registry/factory/dispatch；无 generic catalog。
- `custom_api`: 0.6 deprecation → verified breaking version workflow → 0.7 removal。

同一评论要求 demote/delete public surfaces 先 0.6 deprecate，再 0.7 breaking remove，并提供 CHANGELOG/migration notes。
它只批准 remaining six，不批准历史 66 行的全部 delete/non-LLM lane；T2/T3 保持开放。T10 记录本次
reconciliation，implementation 使用 T11–T23，且保持 T1–T9 历史含义。

## 设计方案

**Phase 1 — 处置矩阵（本 spec 附录，人工批复）**

对 66 个目录逐一标注六类 lane：

- `keep-infra`：base、factory、registry、macros、thinking、openai_like；仅限 shared infra。
- `wired-native`：openai、anthropic、bedrock、mistral、cloudflare、azure、azure_ai、vertex_ai、gemini、
  github_copilot、fal_ai、cohere、replicate 等已有 native enum/factory/dispatch 构造点者。
- `catalog-only-with-native-duplicate`：catalog 已支持但同名 native 目录仍存在者（当前至少 v0 / meta_llama）；
  catalog 条目不能算 native module 可达，必须转为 `demote-to-catalog` 删除 native 目录，或进入显式豁免。
- `demote-to-catalog`：OpenAI 兼容且 catalog runtime 能等价表达的 provider。每个候选必须先证明
  static base URL 足够、无 per-model/dynamic endpoint 构造、无 native-only 非 chat endpoint（如 FIM）、
  auth env fallback 可由 catalog `ProviderDefinition` 表达、`ProviderCapability` / model metadata 与
  native 行为等价；否则进入 wire/delete/exempt 或要求先扩展 catalog 能力。Snowflake、Baseten
  dynamic deployment URL、Codestral FIM、需要 alternate auth env vars 的 Vercel/Codestral 等不能作为
  plain `def()` demote 候选。demote 完成条件必须包含 native 目录删除或显式豁免。
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
- endpoint/auth/capability equivalence evidence：demote 候选必须记录 base_url 是否静态、是否有
  dynamic endpoint 或 provider-specific 非 chat endpoint、primary/alternate auth env vars、native
  capability set 与 catalog `OpenAILikeProvider` capability set 是否等价；
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

**Phase 3 — remaining-six preparation (0.6.0 + Ollama hardening)**

- `amazon_nova`、`github`、`meta_llama`、`v0` 各有一个 provider-scoped policy/equivalence PR。
  测试直接编码上表各自的差异化 contract；同一 PR 加入 0.6.0 deprecation 与兼容性说明，但保留 native module。
- `custom_api` 使用独立 PR 加入 0.6.0 deprecation 和 migration note，保持 public import 可编译。
- `ollama` 先让 ordinary/streaming 请求使用 policy-aware client，覆盖 `api_base`、SSRF、
  private-network authority，移除 `source_boundary_tests.rs` 中的 unwired raw-HTTP exception，并运行
  `scripts/guards/check_outbound_http_clients.sh`；随后才接入 `ProviderType`/registry/factory/dispatch。

**Phase 4 — breaking release gate**

- `.github/workflows/version-bump.yml` 必须显式支持并验证 0.6.x → 0.7.0 breaking bump。
- repository guard 对包含批准 public removals 的 diff 拒绝 patch/minor non-breaking 标记；
  guard tests 只验证 workflow/metadata，不触发真实发布。
- 只有四个 policy/deprecation task 与 `custom_api` deprecation task 都完成后，才执行此 gate。

**Phase 5 — 0.7.0 removal/demotion**

- 四个 catalog provider 各用一个 PR 删除自己的 duplicate native directory、`pub mod`、过时 registry
  metadata并收缩 lifecycle baseline；不得在同一 PR 混入第二个 provider。
- `custom_api` 用一个独立 PR 删除 public/native surface。
- 每 task hard cap 为 500 non-doc changed lines（按 B-006 排除纯删除行，non-doc additions/edits
  绝不豁免），可建议不超过 4 个非纯删除文件。单一 provider directory 的纯删除只可例外物理 file count。
- T5/T6/T9 只汇总 child evidence；T7 closure audit 与 T8 full verification 最后依次执行。

## Appendix A - Historical 66-directory Baseline and Remaining-Six Reconciliation

The 66 rows preserve the original `origin/main@c47596a4` directory baseline, including rows for directories removed
later; they do not assert the current directory count. Only the six amended rows reconcile the 2026-07-15
remaining-six decision with `main@12faaf56`. The short evidence summaries do not complete T2, and the decision
comment does not complete T3 approval for the full delete/non-LLM matrix.

| Directory | Historical/reconciled lane | Evidence summary | Follow-up |
| --- | --- | --- | --- |
| `ai21` | `delete-native` | `providers-extended` public module with `define_pooled_http_provider_with_hooks!`; no native `ProviderType`/factory dispatch. | Delete native directory and `pub mod`, or reclassify after T3. |
| `amazon_nova` | `demote-to-catalog` | Maintainer approved Path A; catalog-backed `ProviderType::AmazonNova` coexists with exported native macro provider. | T11 proves model/pricing/capability policy and adds 0.6 deprecation; T12 demotes only after T9. |
| `anthropic` | `wired-native` | Native `Provider` enum/factory dispatch and literal `LLMProvider` impl. | Keep wired module. |
| `azure` | `wired-native` | `providers-extra` native enum/factory dispatch and literal `LLMProvider` impl. | Keep gated native module. |
| `azure_ai` | `wired-native` | `providers-extra` native enum/factory dispatch and literal `LLMProvider` impl. | Keep gated native module. |
| `base` | `keep-infra` | Shared provider infrastructure; no provider implementation marker. | Keep. |
| `baseten` | `delete-native` | `providers-extended` public module with literal `LLMProvider` impl; no native dispatch; dynamic deployment URL prevents plain catalog demote. | Delete or reclassify after T3. |
| `bedrock` | `wired-native` | Native enum/factory dispatch and literal `LLMProvider` impl. | Keep wired module. |
| `clarifai` | `delete-native` | `providers-extended` public module with literal `LLMProvider` impl; no native dispatch. | Delete or reclassify after T3. |
| `cloudflare` | `wired-native` | Native enum/factory dispatch and literal `LLMProvider` impl. | Keep wired module. |
| `codestral` | `delete-native` | `providers-extended` public module with literal `LLMProvider` impl; no native dispatch; native FIM/auth behavior prevents plain catalog demote. | Delete or reclassify after T3. |
| `cohere` | `wired-native` | `providers-extended` native dispatch and literal `LLMProvider` impl. | Keep gated native module. |
| `custom_api` | `delete-native` | Maintainer decided arbitrary URL/method/template/parser is not a product goal; macro-generated public provider is not shared infra. | T21 deprecates; T22 verifies breaking bump; T9 checks compatibility; T23 removes. |
| `databricks` | `delete-native` | `providers-extended` public module with literal `LLMProvider` impl; no native dispatch. | Delete or reclassify after T3. |
| `datarobot` | `delete-native` | `providers-extended` public module with `define_pooled_http_provider_with_hooks!`; no native dispatch. | Delete or reclassify after T3. |
| `deepgram` | `non-llm-lane` | Audio transcription provider; public module but no `LLMProvider` marker in the guard scan. | Decide non-LLM product support separately. |
| `deepl` | `delete-native` | Translation provider uses `define_http_provider_with_hooks!` and declares `ProviderCapability::ChatCompletion`, so it cannot use the non-LLM lane. | Delete or reclassify after T3. |
| `elevenlabs` | `non-llm-lane` | Text-to-speech/audio transcription provider; public module but no `LLMProvider` marker in the guard scan. | Decide non-LLM product support separately. |
| `empower` | `delete-native` | `providers-extended` public module with `define_pooled_http_provider_with_hooks!`; no native dispatch. | Delete or reclassify after T3. |
| `exa_ai` | `delete-native` | `providers-extended` public module with literal `LLMProvider` impl; no native dispatch. | Delete or reclassify after T3. |
| `factory` | `keep-infra` | Provider construction infrastructure and tests. | Keep. |
| `fal_ai` | `wired-native` | `providers-extended` native dispatch for image generation and literal `LLMProvider` impl. | Keep gated native module. |
| `firecrawl` | `delete-native` | `providers-extended` public module with `define_pooled_http_provider_with_hooks!`; no native dispatch. | Delete or reclassify after T3. |
| `gemini` | `wired-native` | `providers-extended` native dispatch and literal `LLMProvider` impl. | Keep gated native module. |
| `gigachat` | `delete-native` | `providers-extended` public module with literal `LLMProvider` impl; no native dispatch. | Delete or reclassify after T3. |
| `github` | `demote-to-catalog` | Maintainer approved Path A; catalog-backed `ProviderType::GitHub` coexists with exported native provider. | T13 preserves `GITHUB_MODELS_API_BASE` and model/pricing/capability/health; T14 demotes after T9. |
| `github_copilot` | `wired-native` | `providers-extended` native dispatch and literal `LLMProvider` impl. | Keep gated native module. |
| `google_pse` | `delete-native` | Search provider declares `ProviderCapability::ChatCompletion` through an `LLMProvider` surface but has no native LLM dispatch. | Delete or reclassify after T3. |
| `gradient_ai` | `delete-native` | `providers-extended` public module with literal `LLMProvider` impl; no native dispatch. | Delete or reclassify after T3. |
| `huggingface` | `delete-native` | `providers-extended` public module with literal `LLMProvider` impl; no native dispatch. | Delete or reclassify after T3. |
| `jina` | `non-llm-lane` | Embeddings provider exposes literal `LLMProvider` impl. | Decide embedding product lane before delete/wire. |
| `langgraph` | `delete-native` | `providers-extended` public module with literal `LLMProvider` impl; no native dispatch. | Delete or reclassify after T3. |
| `macros` | `keep-infra` | Macro definitions only; guard ignores definitions and scans invocations in provider directories. | Keep. |
| `manus` | `delete-native` | `providers-extended` public module with literal `LLMProvider` impl; no native dispatch. | Delete or reclassify after T3. |
| `meta_llama` | `demote-to-catalog` | Maintainer approved extended provider-scoped catalog policy; exported native module remains under `providers-extra`. | T15 proves auth/identity/filtering/streaming/model/capability equivalence; T16 demotes after T9. |
| `milvus` | `non-llm-lane` | Vector-store provider exposes literal `LLMProvider` impl but is outside LLM factory dispatch. | Decide vector product lane before delete/wire. |
| `mistral` | `wired-native` | Native enum/factory dispatch and literal `LLMProvider` impl. | Keep wired module. |
| `morph` | `delete-native` | `providers-extended` public module with literal `LLMProvider` impl; no native dispatch. | Delete or reclassify after T3. |
| `nlp_cloud` | `delete-native` | `providers-extended` public module with literal `LLMProvider` impl; no native dispatch. | Delete or reclassify after T3. |
| `oci` | `delete-native` | `providers-extended` public module with literal `LLMProvider` impl; no native dispatch. | Delete or reclassify after T3. |
| `ollama` | `wired-native` | Maintainer selected existing native protocol; current source-boundary ledger still exempts its unwired raw-HTTP path. | T19 hardens ordinary/streaming endpoint policy and removes the exception; T20 then wires core dispatch. |
| `openai` | `wired-native` | Native enum/factory dispatch and literal `LLMProvider` impl. | Keep wired module. |
| `openai_like` | `keep-infra` | Shared OpenAI-compatible runtime provider used by explicit and catalog paths. | Keep shared runtime module. |
| `petals` | `delete-native` | `providers-extended` public module with literal `LLMProvider` impl; no native dispatch. | Delete or reclassify after T3. |
| `pg_vector` | `non-llm-lane` | Vector-store module outside LLM factory dispatch and no guard provider marker; tracked by `PROVIDER_ORPHAN_BASELINE`. | Decide vector product lane separately. |
| `predibase` | `delete-native` | `providers-extended` public module with literal `LLMProvider` impl; no native dispatch. | Delete or reclassify after T3. |
| `ragflow` | `delete-native` | `providers-extended` public module with literal `LLMProvider` impl; no native dispatch. | Delete or reclassify after T3. |
| `recraft` | `non-llm-lane` | Image provider exposes literal `LLMProvider` impl. | Decide image product lane before delete/wire. |
| `registry` | `keep-infra` | Catalog, support matrix, lifecycle, and registry metadata. | Keep. |
| `replicate` | `wired-native` | `providers-extended` native dispatch and literal `LLMProvider` impl. | Keep gated native module. |
| `runwayml` | `non-llm-lane` | Video/image provider exposes literal `LLMProvider` impl. | Decide video/image product lane before delete/wire. |
| `sagemaker` | `delete-native` | `providers-extended` public module with literal `LLMProvider` impl; no native dispatch. | Delete or reclassify after T3. |
| `sap_ai` | `delete-native` | `providers-extended` public module with literal `LLMProvider` impl; no native dispatch. | Delete or reclassify after T3. |
| `searxng` | `non-llm-lane` | Search provider exposes literal `LLMProvider` impl. | Decide search product lane before delete/wire. |
| `snowflake` | `delete-native` | `providers-extended` public module with literal `LLMProvider` impl; no native dispatch; endpoint behavior needs more than plain catalog. | Delete or reclassify after T3. |
| `spark` | `delete-native` | `providers-extended` public module with literal `LLMProvider` impl; no native dispatch. | Delete or reclassify after T3. |
| `stability` | `non-llm-lane` | Image provider exposes literal `LLMProvider` impl. | Decide image product lane before delete/wire. |
| `tavily` | `non-llm-lane` | Search provider exposes literal `LLMProvider` impl. | Decide search product lane before delete/wire. |
| `thinking` | `keep-infra` | Shared reasoning trait support. | Keep. |
| `topaz` | `delete-native` | `providers-extended` public module with literal `LLMProvider` impl; no native dispatch. | Delete or reclassify after T3. |
| `triton` | `delete-native` | `providers-extended` public module with literal `LLMProvider` impl; no native dispatch. | Delete or reclassify after T3. |
| `v0` | `demote-to-catalog` | Maintainer approved Path A only after authoritative catalog data replaces the current no-model/zero-cost state; native module remains exported. | T17 proves aliases/model metadata/pricing/health/error policy; T18 demotes after T9. |
| `vercel_ai` | `delete-native` | `providers-extended` public module with literal `LLMProvider` impl; no native dispatch; auth/endpoint behavior requires explicit decision. | Delete or reclassify after T3. |
| `vertex_ai` | `wired-native` | `providers-extra` native dispatch and literal `LLMProvider` impl. | Keep gated native module. |
| `voyage` | `non-llm-lane` | Embedding provider exposes literal `LLMProvider` impl. | Decide embedding product lane before delete/wire. |
| `watsonx` | `delete-native` | `providers-extended` public module with literal `LLMProvider` impl; no native dispatch. | Delete or reclassify after T3. |

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| B-001 approval before deletion | historical matrix / issue approval evidence | T2 full per-row evidence + T3 full-matrix approval; both remain open |
| B-002 native wiring | endpoint policy + enum/factory/registry/dispatch | T19 hardening/source guard, then T20 constructibility |
| B-003 clean deletion | provider dirs / `mod.rs` / lifecycle | all-feature check/test + provider-specific no-reference commands |
| B-004 catalog equivalence | `registry/catalog.rs` + provider policy tests | T11/T13/T15/T17 policy tests, then T12/T14/T16/T18 |
| B-005 persistent guard | `registry/lifecycle_tests.rs` | registry test suite and final T7 audit |
| B-006 bounded deletion | per-provider PR diff | file/line scope evidence in each demotion/removal PR |
| B-007 public API compatibility | deprecation attrs / CHANGELOG / migration / workflow | public API compatibility test + version-bump guard |
| B-008 custom_api lane | `custom_api/**` + lifecycle | T21 deprecation and T23 no-reference/removal checks |
| B-009 non-LLM boundary | capability scan / approved matrix | matrix lane check; no remaining-six task changes non-LLM providers |
| B-010 ordered demotions | task dependencies + per-provider PRs | T11→T9→T12, T13→T9→T14, T15→T9→T16, T17→T9→T18 |
| B-011 provider-specific equivalence | named catalog policy tests | Amazon/GitHub/Meta/V0 assertions in T11/T13/T15/T17 |
| B-012 Ollama native only | `ollama/**`, endpoint policy, source guard, core dispatch | T19 hardening → T20 exact-head safety-gate rerun + wiring/constructibility/catalog absence |
| B-013 custom_api release order | public API test / workflow guard / compatibility / removal | T21→T22→T9→T23 |
| B-014 delivery granularity and closure | PR scope evidence / task graph | B-006-counted ≤500 gate, provider isolation, open T2/T3, then T7→T8 |

## Planned Changes Manifest

| Path or path scope | Intended change | Task ownership |
| --- | --- | --- |
| `src/core/providers/amazon_nova/**` | 0.6 deprecation/equivalence fixtures, then pure native removal | T11, then T12 |
| `src/core/providers/github/**` | 0.6 deprecation/equivalence fixtures, then pure native removal | T13, then T14 |
| `src/core/providers/meta_llama/**` | 0.6 deprecation/equivalence fixtures, then pure native removal | T15, then T16 |
| `src/core/providers/v0/**` | 0.6 deprecation/equivalence fixtures, then pure native removal | T17, then T18 |
| `src/core/providers/ollama/**` | policy-aware ordinary/streaming HTTP, then native wiring/tests | T19, then T20 |
| `src/core/providers/custom_api/**` | 0.6 deprecation, then 0.7 pure native removal | T21, then T23 |
| `src/core/providers/mod.rs` | deprecation exports, Ollama construction surface, later provider export removals | T11–T21, T23 |
| `src/core/providers/base/{http.rs,connection_pool.rs}` | policy-aware request helpers used by Ollama ordinary/streaming paths | T19 |
| `src/core/providers/base/http/source_boundary_tests.rs` | remove Ollama unwired raw-HTTP exception | T19 |
| `src/core/providers/factory/{mod.rs,registry.rs,builder.rs,endpoint_policy.rs,endpoint_access_tests.rs}` | Ollama endpoint authority tests and native construction/dispatch | T19, T20 |
| `src/core/providers/registry/catalog.rs` | four provider-scoped catalog policies/tests; explicit no-Ollama policy | T11, T13, T15, T17 |
| `src/core/providers/registry/{types.rs,support_matrix.rs}` | Ollama `ProviderType`/registry wiring and support metadata | T20 |
| `src/core/providers/registry/{lifecycle.rs,lifecycle_tests.rs}` | shrink duplicate/orphan baselines and preserve guard assertions | T12, T14, T16, T18, T20, T23 |
| `tests/public_api_compat.rs` | compile/deprecation compatibility assertions for 0.6 surfaces | T11, T13, T15, T17, T21 |
| `.github/workflows/version-bump.yml` | explicit breaking 0.7 bump mode and guard invocation | T22 |
| `scripts/guards/check_version_bump.sh` | deterministic, side-effect-free workflow/version policy checks | T22 |
| `CHANGELOG.md` | 0.6 deprecations, 0.7 removals and Ollama user-visible change | T11, T13, T15, T17, T20, T21, T23 |
| `docs/providers/GH837-migration-0.6-to-0.7.md` | migration guidance and alternatives for removed public surfaces | T11, T13, T15, T17, T21, T23 |
| `README.md` | final provider capability synchronization | T7 |
| `CLAUDE.md` | final provider architecture/capability synchronization | T7 |
| `specs/GH837/{product.md,tech.md,tasks.md}` | this task-granularity amendment and later evidence-only status updates | current amendment, T7/T8 |

## 数据流

T19 first moves Ollama ordinary/streaming traffic behind the policy-aware client so configured `api_base` and
private-network authority are revalidated on every path; only then T20 makes `ProviderType::Ollama` constructible
through factory/dispatch. The four catalog policies change metadata/selection only after provider-specific tests;
T9 then gates all duplicate-native removals. `custom_api` remains callable in 0.6 and is removed only after T9.

## 备选方案

- 全部接线（wire all）：~41 个目录补 enum/factory/dispatch，扩大 #519 反对的封闭 enum 面，且多数无用户需求证据，拒绝。
- 保持现状仅加守护：阻止恶化但不解决存量数万行死代码，拒绝。
- 移入独立 `providers-graveyard` feature：仍参与编译与 review 面，拒绝。

## 风险

- Security: policy tests 只验证 env-key contract，不记录真实 token；workflow guard 不执行 release。
- Compatibility: 五个公开 surface 的 removal 是明确的 0.7 breaking change。0.6 deprecation 与 migration
  notes 不得被跳过，且 `github_copilot` 不受 `github` demotion 影响。
- Behavior: catalog policy 缺失数据时存在 silent degradation 风险，尤其 `v0` 的 no-model/zero-cost
  当前状态；因此缺数据必须 hard-fail equivalence，不能 fallback 后继续 demote。
- Performance: duplicate native removal 预期降低编译/review 面；`ollama` wiring 会增加一条可达 native path。
- Maintenance: shared catalog/lifecycle/docs files 形成并行冲突面；coordinator 必须串行安排相交 ownership。

## 测试计划

- [x] Existing guard: conformance 守护测试（含 macro provider、catalog/native duplicate 和 baseline）。
- [ ] Catalog unit tests: 四个 provider 分别测试 B-011，不使用共享的“catalog entry exists”弱断言。
- [ ] Ollama hardening tests: ordinary/streaming policy-aware client、`api_base`、SSRF/private-network authority、
      raw-HTTP exception absence 与 `check_outbound_http_clients.sh`。
- [ ] Native unit/integration tests: hardening 后的 Ollama construct/request/response/streaming，外加 catalog absence 负断言。
- [ ] Compatibility tests: 0.6 public imports 仍编译并带 deprecation；0.7 removal 后无 dangling references。
- [ ] Workflow tests: breaking 0.7 mode 通过，patch/minor disguise 失败，且测试无 release side effect。
- [ ] Full verification: T7 guard/docs closure 后执行 T8 fmt/clippy/test/build/timings。

## 回滚方案

每个 provider/task 一个 PR，因此可逐 provider revert。0.6 policy/deprecation PR 与后续 0.7 removal PR
分离：equivalence 或 workflow gate 失败时不启动 removal，而不是放宽测试。已发布 0.7 的 API removal
不能靠 silent re-export 回滚；需要新版本、CHANGELOG 与 migration update。Ollama hardening/wiring 可按 T20、T19 顺序独立 revert，
不影响四个 catalog policy 或 `custom_api` release chain。
