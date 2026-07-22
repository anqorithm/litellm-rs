# Task Plan

## Linked Issue

GH-1103 / #1103

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## 实现任务

- [ ] `SP1103-T1` Owner: inventory owner | Dependencies: none | Done when: 从 `v0.5.0` tag/package 生成 public API baseline manifest；tracked inventory 按完整 path 覆盖当前 `core::cost`/`core::pricing` exports、production/test consumers、DTO conversions、公开 load/lookup/calculate methods、全部 legacy fallbacks，以及 `pricing_service/authority.rs::provider_catalog_model_info`、Amazon Nova/xAI helper 到 Azure、Bedrock、provider registry/openai-like catalog 的 live authority source，并为每项记录合法 compatibility/authority disposition 与 owner evidence；source guard 对新增 authority branch、漏项和 decoy 失败。 | Verify: baseline manifest fixture；focused inventory/call-graph guard tests；`rg -n "core::cost|core::pricing|PricingDatabase|GLOBAL_PRICING_DB|get_pricing_db|calculate_cost|provider_catalog_model_info|amazon_nova_pricing_model_info|xai_pricing_model_info|CostCalculator" src tests` 与 inventory 人工对账。
- [ ] `SP1103-T2` Owner: maintainer | Dependencies: T1 | Done when: human public-API decision 批准 `core::cost`/`core::pricing` public adapter 的 `keep_adapter`/`deprecate_0_6_remove_0_7` 清单，并对每个 authority-bearing public facade 与 user-visible fallback 批准 `migrate_authority`；`needs_decision` 保持 blocked，且不得仅以 `keep_adapter` 保留独立计算 authority。 | Verify: #1103 或 spec PR 中逐完整 path 的明确批准记录。
- [ ] `SP1103-T3` Owner: compatibility owner | Dependencies: T2 | Done when: 0.6.x tranche 把批准的 user-visible fallback 与 `core::pricing` 独立 lookup/calculation 迁入 `PricingService` authority，compatibility facade 只执行批准的 legacy DTO/error/return-contract 转换；添加 targeted deprecation，保持相对 `v0.5.0` baseline 的 public signature/error/runtime behavior，并同步 CHANGELOG 与迁移文档。 | Verify: tag/package-derived public compile/behavior fixture；priced pricing/spend parity；Azure/Bedrock/Amazon Nova/xAI alias/fallback；默认 `Reject` unknown pricing fail-closed；显式 `AllowUnpriced` configured fallback reservation/settlement parity；`git diff --check`。
- [ ] `SP1103-T4` Owner: release evidence owner | Dependencies: T7a, T3 merged | Done when: 已通过 0.6 final-head verification的 targeted deprecation tranche 合并，且对应 0.6.x release artifact 可验证、migration note 对应 exact public symbols。 | Verify: T7a head-bound gate evidence；release/tag/package evidence与 merged head SHA。
- [ ] `SP1103-T5` Owner: workflow owner | Dependencies: T2 | Done when: version workflow deterministic fixture 证明 0.6.x breaking bump 生成 0.7.0，且不能用非 breaking label 隐藏 removal。 | Verify: focused workflow tests；`python3 checks/check_workflow.py --repo .`。
- [ ] `SP1103-T6` Owner: removal owner | Dependencies: T4, T5, fresh human removal approval | Done when: 独立 0.7.0 breaking tranche 只删除批准且已发布 deprecated 的 `core::cost`/`core::pricing` symbol、adapter/fallback；保留 `PricingService` authority、endpoint/spend behavior、`AllowUnpriced` policy parity 与清单外 provider-local catalog。 | Verify: approved removal manifest；public replacement compile fixture；pricing/spend/provider-policy regressions；exact-diff scope guard。
- [ ] `SP1103-T7a` Owner: 0.6 verification owner | Dependencies: T3 | Done when: 0.6 compatibility head 的 baseline/compile/legacy behavior、authority/facade parity、全部 live provider fallback、默认 `Reject` fail-closed、`AllowUnpriced` reserve/settle parity、SpecRail、format、check、clippy 与 full test 全绿，review threads 为零且 PR gate 满足。 | Verify: 下列 Verification 命令及 0.6 head-bound GitHub evidence。
- [ ] `SP1103-T7b` Owner: 0.7 verification owner | Dependencies: T6 | Done when: 0.7 removal head 的 approved manifest、replacement compile、pricing/spend/provider-policy regression、SpecRail、format、check、clippy 与 full test 全绿，review threads 为零且 PR gate 满足。 | Verify: 下列 Verification 命令及 0.7 head-bound GitHub evidence。
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
- focused inventory/call-graph, pricing authority, `Reject`/`AllowUnpriced` spend parity, Azure/Bedrock/Amazon Nova/xAI fallback, `core::cost`/`core::pricing` public compatibility and version-workflow tests named by each tranche
- `scripts/guards/check_pr_scope.sh`
- `git diff --check`

## Handoff Notes

- 当前 crate version 为 0.5.0；0.6.0 deprecation 是 `implx auto` 默认窗口，不是 removal 授权。
- 本 packet 是 heavy/public-API planning。任何实现前必须有人类批准 T2；0.7 removal 另需 T4/T5 与 fresh approval。
- #726 已完成 user-visible authority convergence，不得把本 issue 描述为重新实现 #726。
- #837 与 #965 有独立 worktree/PR ownership；默认不得修改其 provider/router/registry paths。
- #519 是 roadmap umbrella，不接受本 issue 的 monolithic implementation PR。
