# Task Plan

## Linked Issue

GH-837 / #837

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## 实现任务

- [ ] `SP837-T1` Owner: coordinator. Done when: `specs/GH837/` 三件套通过 SpecRail packet validation. Verify: `SPEC_RAIL=/path/to/specrail; python3 "$SPEC_RAIL/checks/check_workflow.py" --repo "$SPEC_RAIL" --spec-dir "$PWD/specs/GH837"`.
- [ ] `SP837-T2` Owner: coordinator. Done when: 66 目录全量处置矩阵（wire/delete/demote/keep-infra/non-llm-lane + 每行 `rg` 可达性证据）作为附录追加到本 spec. Verify: `git diff -- specs/GH837/`; 每目录一行且附判定命令输出。
- [ ] `SP837-T3` Owner: maintainer. Done when: 维护者在 #837 批复处置矩阵（SpecRail human gate `spec_approval`），特别是 non-llm-lane 的产品定位与 delete 清单. Verify: #837 issue thread 中的明确批复。
- [ ] `SP837-T4` Owner: coordinator. Done when: registry conformance 守护测试合入——扫描 `impl LLMProvider` 与「enum+factory+catalog+豁免清单」求差集，非空失败. Verify: `cargo test core::providers::registry --lib --all-features`; 人为添加孤儿目录的负测试通过验证后移除。
- [ ] `SP837-T5` Owner: coordinator. Done when: delete lane 按 tranche 执行完毕，每 PR 一个目录家族，`pub mod` 与 registry 元数据同步清理. Verify: 每 tranche `cargo check --all-features` + `cargo test --all-features`; `rg -n "pub mod (petals|nlp_cloud|spark|gigachat)" src/core/providers/mod.rs` 无残留（以批复清单为准）。
- [ ] `SP837-T6` Owner: coordinator. Done when: demote lane 执行完毕，每 provider 一个 PR（catalog `def()` + 删目录 + smoke 等价验证）. Verify: `cargo test core::providers::registry::catalog --lib --all-features`; 每 PR 附 base_url/env-key 对照。
- [ ] `SP837-T7` Owner: verification owner. Done when: 收尾扫描确认无不可达 `impl LLMProvider`（豁免清单除外），README / CLAUDE.md provider 描述已同步. Verify: conformance 测试绿色; `rg -c "impl LLMProvider" src/core/providers | wc -l` 与矩阵一致。

## 并行拆分

- SP837-T4（守护测试，只动 `registry/`）与 SP837-T2（纯文档附录）可并行。
- SP837-T5 各 tranche 之间文件不相交，可多 lane 并行（W-14：每 tranche 独占自己的目录集）。
- SP837-T6 依赖 T3 批复；T5/T6 均依赖 T4 先合入（守护测试作为删除的安全网）。

## 验证

- [ ] `SP837-T8` Owner: verification owner. Done when: 全部 tranche 合并后全量验证通过且编译时间对比有记录. Verify: `cargo fmt --all -- --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-features`; `cargo build --all-features --timings` 前后对比。

## Handoff Notes

- U-05 约束：任何目录在 SP837-T3 批复前不得删除；「看起来没用」不是删除依据，零构造路径 + 维护者批复才是。
- non-llm-lane（搜索/向量/语音 9 个目录）本质是产品范围问题：litellm-rs 是否要做非 LLM 网关能力。建议维护者在 #837 一并表态。
- 与 #519 A-4 的边界：本 issue 不改 dispatch 架构；若 A-4 先落地（enum → trait object），wire lane 的接线方式随之变化，处置矩阵仍然有效。
