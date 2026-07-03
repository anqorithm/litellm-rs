# Task Plan

## Linked Issue

GH-837 / #837

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## 实现任务

- [ ] `SP837-T1` Owner: coordinator. Done when: `specs/GH837/` 三件套通过 SpecRail packet validation. Verify: `SPEC_RAIL=/path/to/specrail; python3 "$SPEC_RAIL/checks/check_workflow.py" --repo "$SPEC_RAIL" --spec-dir "$PWD/specs/GH837"`.
- [ ] `SP837-T2` Owner: coordinator. Done when: 66 目录全量处置矩阵（wired-native/delete-native/demote-to-catalog/keep-infra/non-llm-lane/exempt + 每行 construction/dispatch、catalog、public export、macro、capability 证据）作为附录追加到本 spec. Verify: `git diff -- specs/GH837/`; 每目录一行且附判定命令输出，裸 `rg "<TypeName>" src` 文本命中不得作为可达性证据。
- [ ] `SP837-T3` Owner: maintainer. Done when: 维护者在 #837 批复处置矩阵（SpecRail human gate `spec_approval`），特别是 non-llm-lane 的产品定位与 delete 清单. Verify: #837 issue thread 中的明确批复。
- [ ] `SP837-T4` Owner: coordinator. Done when: registry conformance 守护测试合入——扫描 literal `impl LLMProvider`、`define_http_provider_with_hooks!`、`define_pooled_http_provider_with_hooks!` 等 macro-generated provider，并与「native enum/factory/dispatch + catalog-only 完成状态 + 维护者批复的临时 orphan baseline + 豁免清单」求差集，新增非 baseline orphan 即失败；catalog 条目不得抵消仍存在的重复 native module. Verify: `cargo test core::providers::registry --lib --all-features`; 人为添加 literal impl、macro provider、pooled macro provider、catalog/native duplicate 四类孤儿 fixture 的负测试通过验证后移除。
- [ ] `SP837-T5` Owner: coordinator. Done when: delete-native lane 按 tranche 执行完毕，每 PR 一个目录家族，`pub mod` 与 registry 元数据同步清理；image/video/translation/search/vector/embedding-only provider 不进入此 lane，除非 non-LLM 产品决策明确批准. Verify: 每 tranche `cargo check --all-features` + `cargo test --all-features`; `rg -n "pub mod (petals|nlp_cloud|spark|gigachat)" src/core/providers/mod.rs` 无残留（以批复清单为准）。
- [ ] `SP837-T6` Owner: coordinator. Done when: demote-to-catalog lane 执行完毕，每 provider 一个 PR（catalog `def()` + 删 native 目录 + smoke 等价验证）；若 native 目录暂留，必须进入带 issue/owner/期限的豁免清单. Verify: `cargo test core::providers::registry::catalog --lib --all-features`; 每 PR 附 base_url/env-key 对照与无重复 native impl 证据。
- [ ] `SP837-T7` Owner: verification owner. Done when: 收尾扫描确认无不可达 `impl LLMProvider`（豁免清单除外），README / CLAUDE.md provider 描述已同步. Verify: conformance 测试绿色; `rg -c "impl LLMProvider" src/core/providers | wc -l` 与矩阵一致。
- [ ] `SP837-T9` Owner: coordinator. Done when: 删除任何 `pub mod` provider 前完成 public API / semver 影响表，标明 refactor、deprecation、或 breaking-change 路径. Verify: 每个 delete/demote tranche PR 描述或 CHANGELOG 草案包含 compatibility 决策；`src/core/providers/mod.rs` 删除项均能追溯到矩阵行。

## 并行拆分

- SP837-T4（守护测试，只动 `registry/`）与 SP837-T2（纯文档附录）可并行。
- SP837-T5 各 tranche 之间文件不相交，可多 lane 并行（W-14：每 tranche 独占自己的目录集）。
- SP837-T5/T6 依赖 T3 批复与 T9 compatibility 决策；T4 可在 T5/T6 前合入，但必须把 T3 批复矩阵中的当前 orphan 列为带 issue/owner/期限的临时 baseline，只对新增或未批准 orphan hard-fail。T5/T6 完成后逐步清空 baseline。

## 验证

- [ ] `SP837-T8` Owner: verification owner. Done when: 全部 tranche 合并后全量验证通过且编译时间对比有记录. Verify: `cargo fmt --all -- --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-features`; `cargo build --all-features --timings` 前后对比。

## Handoff Notes

- U-05 约束：任何目录在 SP837-T3 批复前不得删除；「看起来没用」不是删除依据，零构造路径 + 维护者批复才是。
- catalog 条目只能证明 `Provider::OpenAILike` 路径；当同名 native module 仍存在（如 v0 / meta_llama）时，
  不能把 catalog 计为 native impl 可达。
- `custom_api` 是 macro-generated provider，不是 shared infra；必须 wire/delete/demote/exempt 明确归类。
- non-llm-lane（搜索/向量/语音/image/video/translation/embedding-only 目录）本质是产品范围问题：
  litellm-rs 是否要做非 LLM 网关能力。建议维护者在 #837 一并表态。
- 可达性证据只接受 construction/dispatch/symbol/capability 证据；文档、注释、tests 或无关同名 raw text hit 只能作为旁证。
- 与 #519 A-4 的边界：本 issue 不改 dispatch 架构；若 A-4 先落地（enum → trait object），wire lane 的接线方式随之变化，处置矩阵仍然有效。
