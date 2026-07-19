# Task Plan

## Linked Issue

GH-837 / #837

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`
- Remaining-six authority: <https://github.com/majiayu000/litellm-rs/issues/837#issuecomment-4982855968>

## 历史任务（T1–T9 ID/含义不变）

- [x] `SP837-T1` Owner: coordinator. Covers: B-001. Dependencies: none. Done when: `specs/GH837/` 三件套通过 SpecRail packet validation. Verify: `env PYTHONDONTWRITEBYTECODE=1 python3 checks/check_workflow.py --repo . --spec-dir specs/GH837`.
- [ ] `SP837-T2` Owner: coordinator. Covers: B-001, B-004, B-008, B-009. Dependencies: T1. Done when: 历史 66 目录 baseline 的每行均有 construction/dispatch、catalog、public export、macro、capability、internal dependency/metadata-use、endpoint/auth/capability equivalence 完整证据. Verify: 运行 Appendix 66-row check 并逐行核验 evidence ledger；当前短摘要不满足 done-when。
- [ ] `SP837-T3` Owner: maintainer. Covers: B-001. Dependencies: T2. Done when: 维护者批准历史 66 行全量处置矩阵，特别是全部 delete 与 non-LLM lane. Verify: #837 issue thread 中明确覆盖全矩阵的批复；comment `4982855968` 只批准 remaining six，不满足本 task。
- [x] `SP837-T4` Owner: coordinator. Covers: B-005. Dependencies: T1. Done when: registry conformance guard 扫描 literal/macro provider、duplicate catalog/native 和有期限 baseline，新增 orphan 会失败. Verify: `git merge-base --is-ancestor c4f5e9f7 HEAD && cargo test core::providers::registry --lib --all-features`.
- [ ] `SP837-T5` Owner: coordinator. Covers: B-003, B-006, B-008, B-009. Dependencies: T2, T3, T23. Done when: 原 delete-native lane 按 tranche 完成，每 PR 一个目录家族并同步 `pub mod`/registry；internal dependency 先迁移；未经产品批准不混入 non-LLM/chat-capable adapter. Verify: 每 tranche `cargo check --all-features && cargo test --all-features`，且 `custom_api` 的 T23 removal evidence 完整。
- [ ] `SP837-T6` Owner: coordinator. Covers: B-004, B-006, B-007, B-010, B-011. Dependencies: T2, T3, T12, T14, T16, T18. Done when: 原 demote lane 每 provider 一个 PR，获批 catalog route 与 endpoint/auth/capability 等价，native directory 删除或进入时限豁免；remaining four child removals 完成. Verify: `for p in amazon_nova github meta_llama v0; do test ! -d "src/core/providers/$p" || exit 1; done; cargo test core::providers::registry::catalog --lib --all-features`.
- [ ] `SP837-T7` Owner: verification owner. Covers: B-002, B-003, B-005, B-008, B-012, B-014. Dependencies: T2, T3, T5, T6, T9, T20. Done when: 最终扫描无不可达 `LLMProvider`（批准豁免除外），baseline 与 README/CLAUDE 已按最终矩阵同步. Verify: `cargo test core::providers::registry --lib --all-features && rg -n 'amazon_nova|github|meta_llama|v0|ollama|custom_api' README.md CLAUDE.md src/core/providers/registry/lifecycle.rs`.
- [ ] `SP837-T9` Owner: coordinator. Covers: B-007, B-010, B-013. Dependencies: T11, T13, T15, T17, T21, T22. Done when: 所有计划删除的 public surfaces 已在 0.6 deprecate，compatibility table/CHANGELOG/migration 与 breaking workflow evidence 完整；此 gate 必须先于任何 removal. Verify: `rg -n 'amazon_nova|github|meta_llama|v0|custom_api' CHANGELOG.md docs/providers && bash scripts/guards/check_version_bump.sh`.

## Remaining-six amendment and implementation

Hard cap：每 task ≤500 non-doc changed lines（按 B-006 排除纯删除行，non-doc additions/edits 绝不豁免）；可建议 ≤4 个非纯删除文件。单一 provider 的纯删除只可例外物理 file count；一个 task/PR 不得混入第二 provider。

- [x] `SP837-T10` Owner: coordinator. Covers: B-010, B-011, B-012, B-013, B-014. Dependencies: T1, T4. Done when: remaining-six maintainer decision 已与 `main@12faaf56` 当前 packet/source reconciliation，且未冒充全 66 matrix approval. Verify: `gh api repos/majiayu000/litellm-rs/issues/comments/4982855968 --jq '[.user.login,.author_association,.created_at,.html_url,.body] | @tsv' && env PYTHONDONTWRITEBYTECODE=1 python3 checks/check_workflow.py --repo . --spec-dir specs/GH837`.
- [ ] `SP837-T11` Owner: amazon_nova provider owner. Covers: B-004, B-007, B-010, B-011, B-014. Dependencies: T10. Done when: catalog model/pricing/capability policy/equivalence 与 0.6 deprecation/notes 完成，native 保留. Verify: `test -d src/core/providers/amazon_nova && cargo test --lib --all-features amazon_nova_catalog_policy && rg -n '0\.6\.0|deprecat' src/core/providers/amazon_nova CHANGELOG.md docs/providers`.
- [ ] `SP837-T12` Owner: amazon_nova provider owner. Covers: B-003, B-004, B-006, B-007, B-010, B-014. Dependencies: T11, T9. Done when: 仅该 provider native surface 在 0.7 删除且 baseline 收缩，catalog tests 仍绿. Verify: `test ! -d src/core/providers/amazon_nova && ! rg -n 'pub mod amazon_nova|AmazonNovaProvider' src/core/providers/mod.rs src/core/providers/registry && cargo test --lib --all-features amazon_nova`.
- [ ] `SP837-T13` Owner: github provider owner. Covers: B-004, B-007, B-010, B-011, B-014. Dependencies: T10. Done when: catalog 保留 `GITHUB_MODELS_API_BASE`、model/pricing/capability/health 且完成 0.6 deprecation/notes，native 保留. Verify: `test -d src/core/providers/github && cargo test --lib --all-features github_catalog_policy && rg -n 'GITHUB_MODELS_API_BASE|0\.6\.0|deprecat' src/core/providers/github src/core/providers/registry/catalog.rs CHANGELOG.md docs/providers`.
- [ ] `SP837-T14` Owner: github provider owner. Covers: B-003, B-004, B-006, B-007, B-010, B-014. Dependencies: T13, T9. Done when: 仅 `github` native surface 在 0.7 删除，`github_copilot` 不变. Verify: `test ! -d src/core/providers/github && test -d src/core/providers/github_copilot && ! rg -n 'pub mod github;' src/core/providers/mod.rs && cargo test --lib --all-features github_catalog_policy`.
- [ ] `SP837-T15` Owner: meta_llama provider owner. Covers: B-004, B-007, B-010, B-011, B-014. Dependencies: T10. Done when: catalog auth/identity/filtering/streaming/model metadata/capability equivalence 与 0.6 deprecation/notes 完成，native 保留. Verify: `test -d src/core/providers/meta_llama && cargo test --lib --all-features meta_llama_catalog_policy && rg -n '0\.6\.0|deprecat' src/core/providers/meta_llama CHANGELOG.md docs/providers`.
- [ ] `SP837-T16` Owner: meta_llama provider owner. Covers: B-003, B-004, B-006, B-007, B-010, B-014. Dependencies: T15, T9. Done when: 仅该 provider native surface 在 0.7 删除且 policy tests 仍绿. Verify: `test ! -d src/core/providers/meta_llama && ! rg -n 'pub mod meta_llama|MetaLlamaProvider' src/core/providers/mod.rs src/core/providers/registry && cargo test --lib --all-features meta_llama_catalog_policy`.
- [ ] `SP837-T17` Owner: v0 provider owner. Covers: B-004, B-007, B-010, B-011, B-014. Dependencies: T10. Done when: authoritative aliases/model metadata/pricing/health/error policy 拒绝 no-model/zero-cost canonical fallback，并完成 0.6 deprecation/notes，native 保留. Verify: `test -d src/core/providers/v0 && cargo test --lib --all-features v0_catalog_policy && rg -n 'aliases|pricing|health|error|0\.6\.0|deprecat' src/core/providers/v0 src/core/providers/registry/catalog.rs CHANGELOG.md docs/providers`.
- [ ] `SP837-T18` Owner: v0 provider owner. Covers: B-003, B-004, B-006, B-007, B-010, B-014. Dependencies: T17, T9. Done when: 仅该 provider native surface 在 0.7 删除且 authoritative policy tests 仍绿. Verify: `test ! -d src/core/providers/v0 && ! rg -n 'pub mod v0|V0Provider' src/core/providers/mod.rs src/core/providers/registry && cargo test --lib --all-features v0_catalog_policy`.
- [ ] `SP837-T19` Owner: ollama provider owner. Covers: B-002, B-012, B-014. Dependencies: T10. Done when: 普通与 streaming Ollama 请求均使用 policy-aware client；`api_base`、SSRF 与 private-network authority tests 完整；移除 `ollama/provider.rs` unwired raw-HTTP exception. Verify: `cargo test --lib --all-features ollama && cargo test --lib --all-features endpoint_access && ! rg -n 'src/core/providers/ollama/provider.rs' src/core/providers/base/http/source_boundary_tests.rs && bash scripts/guards/check_outbound_http_clients.sh`.
- [ ] `SP837-T20` Owner: ollama provider owner. Covers: B-002, B-012, B-014. Dependencies: T19. Done when: hardened native protocol 接入 core `ProviderType`/registry/factory/dispatch 并覆盖 request/response/streaming，无 generic catalog route. Verify: wiring exact head 必须重跑 `cargo test --lib --all-features ollama && cargo test --lib --all-features endpoint_access && ! rg -n 'src/core/providers/ollama/provider.rs' src/core/providers/base/http/source_boundary_tests.rs && bash scripts/guards/check_outbound_http_clients.sh`，再运行 `rg -n 'Ollama' src/core/providers/{mod.rs,factory,registry} && ! rg -n 'def\([^\n]*ollama|provider_id: "ollama"' src/core/providers/registry/catalog.rs`.
- [ ] `SP837-T21` Owner: custom_api provider owner. Covers: B-007, B-008, B-013, B-014. Dependencies: T10. Done when: public module/types 在 0.6 deprecate 但仍可编译，notes 说明任意 URL/method/template/parser 不再是产品目标. Verify: `test -d src/core/providers/custom_api && cargo test --test public_api_compat custom_api_deprecated_in_0_6 && rg -n 'custom_api|0\.6\.0|deprecat' src/core/providers/custom_api src/core/providers/mod.rs CHANGELOG.md docs/providers`.
- [ ] `SP837-T22` Owner: release workflow owner. Covers: B-007, B-010, B-013, B-014. Dependencies: T11, T13, T15, T17, T21. Done when: version workflow 显式验证 0.6.x→0.7.0 breaking bump并拒绝把 public removal 伪装成 non-breaking，测试无发布副作用. Verify: `bash scripts/guards/check_version_bump.sh && ruby -e 'require "yaml"; YAML.safe_load(File.read(".github/workflows/version-bump.yml"), aliases: true)'`.
- [ ] `SP837-T23` Owner: custom_api provider owner. Covers: B-003, B-006, B-007, B-008, B-013, B-014. Dependencies: T21, T9. Done when: 0.7 删除该 public/native surface、registry/lifecycle entries，并有 migration alternative. Verify: `test ! -d src/core/providers/custom_api && ! rg -n 'custom_api|CustomApi' src/core/providers/mod.rs src/core/providers/registry && cargo check --all-features && cargo test --all-features`.

## 执行顺序

- T11/T13/T15/T17/T21 在 T10 后准备 0.6 compatibility；共享 catalog/docs files 时串行。
- T22 在上述五个 0.6 tasks 后执行；T9 汇总并验证 compatibility evidence。
- T12/T14/T16/T18/T23 仅在 T9 后 removal；T19 hardening 完成后才可 T20 Ollama wiring。
- T5/T6 仍被开放 T2/T3 fail-closed；T7 closure 后才执行 T8。

## 验证

- [ ] `SP837-T8` Owner: verification owner. Covers: B-003, B-005, B-014. Dependencies: T2, T3, T7. Done when: 全部 tranche 合并后全量验证通过且编译时间对比有记录. Verify: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-features && cargo build --all-features --timings`.

## Handoff Notes

- 缺 catalog data 等于“不满足等价”，不得以空 model/zero pricing 或猜测值降级。
- 每 task/PR 只处理一个 provider；`github` 与 `github_copilot` 是不同 scope。
- T2/T3/T5/T6/T7/T8 开放期间不得把 GH837 标记 complete。
