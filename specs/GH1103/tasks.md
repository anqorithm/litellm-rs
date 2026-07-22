# Task Plan

## Linked Issue

GH-1103 / #1103

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## 实现任务

- [ ] `SP1103-T1` Owner: inventory owner | Dependencies: none | Done when: 从 `v0.5.0@de594c81` tag/package 分别在 default features 与 docs.rs exact set `gateway,postgres,sqlite,redis,s3,metrics,tracing,websockets,analytics,providers-extra,providers-extended` 生成 public API baseline manifest；published cohort 精确覆盖 `core::cost`、`core::providers::base::pricing`、`utils::ModelUtils::get_model_pricing`、`utils::TokenUtils::calculate_cost` 的全部 public re-export/module path，以及 feature-gated Azure/Bedrock 等 provider pricing/cost API，并证明 tag 不含 `src/core/pricing.rs`；tracked current-head inventory 另覆盖 post-v0.5 `core::pricing`、全部 legacy/live fallback 与 `provider_catalog_model_info`→Azure/Bedrock/Amazon Nova/xAI source，为每项记录 baseline status、feature lane、compatibility/authority disposition 与 owner evidence；source guard 对 feature/cohort 混淆、新增 authority branch、漏项和 decoy 失败。 | Verify: two-lane tag/package baseline fixtures；focused inventory/call-graph guard tests；`git cat-file -e v0.5.0:src/core/providers/base/pricing.rs && ! git cat-file -e v0.5.0:src/core/pricing.rs`；tag `Cargo.toml` docs.rs feature assertion；`rg -n "core::cost|core::providers::base|core::pricing|ModelUtils|TokenUtils|get_model_pricing|calculate_cost|provider_catalog_model_info|amazon_nova_pricing_model_info|xai_pricing_model_info|CostCalculator" src tests` 与 inventory 人工对账。
- [ ] `SP1103-T2` Owner: maintainer | Dependencies: T1 | Done when: human public-API decision 仅对 v0.5 published core/provider-base/utility/feature-gated provider adapter 批准 `keep_adapter`/`deprecate_0_6_remove_0_7` 清单；post-v0.5 `core::pricing` 记录 `post_v0_5_unreleased` 而非 published disposition，并对 `ModelUtils::get_model_pricing`、`TokenUtils::calculate_cost`、每个 current-head authority-bearing facade 与全部 user-visible fallback 批准 `migrate_authority`；`needs_decision` 保持 blocked，且不得仅以 re-export、current-head public 或 `keep_adapter` 保留独立计算 authority。 | Verify: #1103 或 spec PR 中逐完整 path、带 baseline/feature cohort 的明确批准记录。
- [ ] `SP1103-T3` Owner: compatibility owner | Dependencies: T2 | Done when: 0.6.x tranche 把批准的 utility pricing、user-visible fallback 与 post-v0.5/current-head `core::pricing` 独立 lookup/calculation 迁入 `PricingService` authority，compatibility facade 只执行批准的 legacy DTO/error/tuple/return-contract 转换；只对 v0.5 published core/provider-base/utility/feature-gated cohort 添加 targeted deprecation并保持其 public signature/error/runtime behavior，不能给 `core::pricing` 伪造 v0.5 compatibility gate；live consumers 使用 configured/loaded runtime source，v0.5-signature facade 只使用 embedded source，未经批准不引入 authority injection；同步 CHANGELOG 与迁移文档。 | Verify: default + exact docs.rs feature tag/package compile/behavior fixtures（实际调用 utility、Azure 与 Bedrock pricing API）；独立 current-head `core::pricing` authority disposition fixture；sentinel custom-source pricing route/reservation/settlement parity；embedded authority/facade parity；禁止跨 source equality assertion；Azure/Bedrock/Amazon Nova/xAI alias/fallback；默认 `Reject` unknown pricing fail-closed；显式 `AllowUnpriced` configured fallback reservation/settlement parity；`git diff --check`。
- [ ] `SP1103-T4` Owner: release evidence owner | Dependencies: T7a, T3 merged | Done when: 已通过 0.6 final-head verification的 targeted deprecation tranche 合并，且对应 0.6.x release artifact 可验证、migration note 对应 exact public symbols。 | Verify: T7a head-bound gate evidence；release/tag/package evidence与 merged head SHA。
- [ ] `SP1103-T5` Owner: workflow owner | Dependencies: T2 | Done when: version workflow deterministic fixture 证明 0.6.x breaking bump 生成 0.7.0，且不能用非 breaking label 隐藏 removal。 | Verify: focused workflow tests；`python3 checks/check_workflow.py --repo .`。
- [ ] `SP1103-T6` Owner: removal owner | Dependencies: T4, T5, fresh human removal approval | Done when: 独立 0.7.0 breaking tranche 只删除 v0.5 published cohort 中批准且已在 0.6 发布为 deprecated 的 core/provider-base/utility/feature-gated provider pricing symbol、adapter/fallback；post-v0.5 `core::pricing` 不因 current-head inventory 自动进入该 removal 清单；保留 `PricingService` source semantics、endpoint/spend behavior、`AllowUnpriced` policy parity 与清单外 provider-local catalog。 | Verify: approved published-cohort removal manifest；default + exact docs.rs feature public replacement compile fixture；source-aware pricing/spend/provider-policy regressions；exact-diff scope guard。
- [ ] `SP1103-T7a` Owner: 0.6 verification owner | Dependencies: T3 | Done when: 0.6 compatibility head 的 v0.5 default/docs.rs published-cohort baseline/compile/legacy behavior、独立 current-head `core::pricing` authority disposition、sentinel custom-source live parity、embedded compatibility parity、全部 live provider fallback、默认 `Reject` fail-closed、`AllowUnpriced` reserve/settle parity、SpecRail、format、check、clippy 与 full test 全绿，review threads 为零且 PR gate 满足。 | Verify: 下列 Verification 命令及 0.6 head-bound GitHub evidence。
- [ ] `SP1103-T7b` Owner: 0.7 verification owner | Dependencies: T6 | Done when: 0.7 removal head 的 approved default/docs.rs v0.5 published-cohort manifest、two-lane replacement compile、source-aware pricing/spend/provider-policy regression、SpecRail、format、check、clippy 与 full test 全绿，review threads 为零且 PR gate 满足。 | Verify: 下列 Verification 命令及 0.7 head-bound GitHub evidence。
- [ ] `SP1103-T8` Owner: roadmap owner | Dependencies: T2 | Done when: #519 记录 A-3 被 #729 决策 supersede、A-4 由 #965 承接、A-6 由 #1103 承接；只有 GH1103 全部 closure criteria 满足后才关闭 A-6 ownership。 | Verify: #519 live issue comment/state 与 child links。

## 并行拆分

- T1 inventory/guard 必须先串行完成，T2 maintainer decision 依赖其完整证据。
- T3 compatibility 与 T5 version-workflow 可在 T2 后分成不重叠 worktree：T3 只拥有批准的 cost/pricing/docs/test paths，T5 只拥有 version workflow/tests；若共享 CHANGELOG 或 guard manifest 则串行。
- T4 是发布证据门禁，不与代码实现并行假定完成。
- T6 必须在 T4/T5 与 fresh human approval 后单独执行，禁止与 0.6 tranche 混合。
- T7a/T7b reviewer/verification lane 分别只读；coordinator 在对应 final head 各独占一次 full verification。

## 验证

- `python3 checks/check_workflow.py --repo .`
- `python3 checks/check_workflow.py --repo . --spec-dir specs/GH1103`
- `cargo fmt --all -- --check`
- `cargo check --all-targets --all-features --locked`
- `cargo clippy --all-targets --all-features --locked -- -D warnings`
- `cargo test --all-features --locked -- --test-threads=1`
- focused v0.5 default/docs.rs core/provider-base/utility/feature-gated compatibility baseline, post-v0.5 `core::pricing` current-head authority inventory, source-aware custom/embedded parity, call-graph, `Reject`/`AllowUnpriced` spend parity, Azure/Bedrock/Amazon Nova/xAI fallback and version-workflow tests named by each tranche
- `scripts/guards/check_pr_scope.sh`
- `git diff --check`

## Handoff Notes

- 当前 crate version 为 0.5.0；0.6.0 deprecation 是 `implx auto` 默认窗口，不是 removal 授权。
- 本 packet 是 heavy/public-API planning。任何实现前必须有人类批准 T2；0.7 removal 另需 T4/T5 与 fresh approval。
- #726 已完成 user-visible authority convergence，不得把本 issue 描述为重新实现 #726。
- #837 与 #965 有独立 worktree/PR ownership；默认不得修改其 provider/router/registry paths。
- #519 是 roadmap umbrella，不接受本 issue 的 monolithic implementation PR。
