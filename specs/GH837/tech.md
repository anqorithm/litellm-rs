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
| Tier-1 catalog | `src/core/providers/registry/catalog.rs` | 数据驱动 OpenAI 兼容 provider → `Provider::OpenAILike` | demote 目标位置 |
| Orphan dirs | `src/core/providers/{deepgram,ollama,elevenlabs,stability,huggingface,sagemaker,watsonx,voyage,databricks,triton,jina,...}` | `pub mod` 声明 + 完整实现，目录外零引用 | 处置对象（~41 个） |
| Registry types | `src/core/providers/registry/{types.rs,lifecycle.rs,support_matrix.rs}` | `PROVIDER_TYPE_REGISTRY` 等元数据 | 守护测试挂载点 |
| Prior art | #137 / #140 / #714（均 CLOSED） | 清理过一轮后回归 | 说明需要守护测试 |

## 设计方案

**Phase 1 — 处置矩阵（本 spec 附录，人工批复）**

对 66 个目录逐一标注五类 lane：

- `keep-infra`：base、factory、registry、macros、thinking、custom_api、openai_like。
- `wired`：openai、anthropic、bedrock、mistral、cloudflare、azure、azure_ai、vertex_ai、gemini、
  github_copilot、fal_ai、cohere、replicate（+ 确认 v0 / meta_llama 实际状态）。
- `demote-to-catalog`：OpenAI 兼容且无自定义流式/鉴权者（候选：baseten、codestral、empower、
  datarobot、gradient_ai、morph、predibase、snowflake、vercel_ai 等，逐个验证 API 形状）。
- `delete`：非 OpenAI 兼容、无用户需求证据、不可达者（候选：petals、nlp_cloud、spark、gigachat、
  clarifai、ragflow、sap_ai、topaz、runwayml、recraft 等）。
- `non-llm-lane`：tavily、searxng、google_pse、exa_ai、firecrawl（搜索/工具）、milvus、pg_vector
  （向量库）、deepgram、elevenlabs（语音）——先决定产品上是否保留这些能力，再决定 wire/delete。

判定脚本（附录附命令）：对每个目录 `rg "<TypeName>" src --glob '!src/core/providers/<dir>/**'`，
零命中即不可达；结果表进附录。

**Phase 2 — 守护测试（先行合入）**

在 `registry` 增加 conformance 测试：扫描 `src/core/providers/*/` 的 `impl LLMProvider` 类型名，
与「enum 变体 + factory 分支 + catalog 条目 + 豁免清单」求差集，非空即失败。豁免清单为带
issue 引用的常量表，CI 可见。

**Phase 3 — 分批执行**

- delete lane：按目录家族分 tranche（每 PR 一个或数个小目录），纯删除 + `pub mod` 清理。
- demote lane：每 PR 一个 provider：加 catalog `def()` → 删目录 → smoke 验证模型列表/鉴权头等价。
- wire lane（如维护者选择保留个别）：按 CLAUDE.md Tier-2 流程补 enum/factory/dispatch。

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P2 wire 可达 | factory/enum | conformance 测试 + 单测构造 |
| P3 delete 干净 | providers/mod.rs | `cargo check --all-features` + `rg` 无 dangling mod |
| P4 demote 等价 | catalog.rs | catalog smoke 测试（base_url/env key/名称） |
| P5 守护常驻 | registry conformance test | CI 上人为引入孤儿目录的负测试 |

## 数据流

无运行时数据流变化；delete/demote 仅移除不可达代码路径。

## 备选方案

- 全部接线（wire all）：~41 个目录补 enum/factory/dispatch，扩大 #519 反对的封闭 enum 面，且多数无用户需求证据，拒绝。
- 保持现状仅加守护：阻止恶化但不解决存量数万行死代码，拒绝。
- 移入独立 `providers-graveyard` feature：仍参与编译与 review 面，拒绝。

## 风险

- Security: 无；删除减少攻击面。
- Compatibility: 若有用户依赖未文档化的孤儿目录（不可能——无构造路径），风险为零；demote 需逐个验证 API 形状。
- Performance: 编译时间预期显著下降（删除数万行 + 各目录 tests）。
- Maintenance: 主要风险是误删 keep-infra 依赖，靠 `cargo check --all-features` 与全量测试兜底。

## 测试计划

- [ ] Unit tests: conformance 守护测试（含豁免清单机制）。
- [ ] Integration tests: demote 后 catalog smoke 测试。
- [ ] Manual verification: 处置矩阵逐行与 `rg` 输出核对。

## 回滚方案

删除类 PR 逐个 revert 即可恢复；git history 保留全部实现。守护测试可通过豁免清单临时放行。
