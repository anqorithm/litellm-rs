# Task Plan

## Linked Issue

GH-838 / #838

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## 实现任务

- [ ] `SP838-T1` Owner: coordinator. Done when: `specs/GH838/` 三件套通过 SpecRail packet validation. Verify: `SPEC_RAIL=/path/to/specrail; python3 "$SPEC_RAIL/checks/check_workflow.py" --repo "$SPEC_RAIL" --spec-dir "$PWD/specs/GH838"`.
- [ ] `SP838-T2` Owner: coordinator. Done when: 可达性证据表（每子系统的 `rg` 命中数、依赖、测试规模、churn）作为附录追加到本 spec，范围包含 audit logging 与 webhooks. Verify: `git diff -- specs/GH838/`; 每子系统一行且附判定命令。
- [ ] `SP838-T3` Owner: maintainer. Done when: 维护者在 #838 批复处置矩阵（wire/remove/experimental-gate），特别是 mcp/a2a/realtime 的产品意向与 guardrails 默认开关（SpecRail human gate `spec_approval`）. Verify: #838 issue thread 中的明确批复。
- [ ] `SP838-T4` Owner: coordinator. Done when: 守护检查合入 CI——`src/core/mod.rs` 顶层 `pub mod` 先分类为 gateway-facing / library-only / internal-support / feature-gated；gateway-facing 模块必须被启动装配实际构造并接入请求路径/中间件/路由/后台任务，或在豁免清单，或被真 default-off feature gate；default-on support feature（如 storage/sqlite）不算 experimental gate，config/admin/status 文本引用不算可达. Verify: 检查脚本/测试绿色；人为添加 config-only/admin-only 未接线 gateway 模块的负测试验证后移除；人为把 gateway-facing 模块只挂到 default-on support feature 的负测试会失败；`completion`、`function_calling`、`traits`、`secret_managers` 作为 library-only 正测试不被误拦。
- [ ] `SP838-T5` Owner: coordinator. Done when: wire lane 执行完毕——每子系统一个 PR，含 `GatewayConfig` 字段、启动初始化、中间件/路由挂载、smoke 测试（U-26 三要件）；observability+integrations 还必须在真实 LLM request 生命周期触发 `on_llm_start` 与 `on_llm_end`/`on_llm_error`. Verify: 每 PR `cargo test --all-features` + 冒烟请求记录；observability PR 用 test integration 或 Langfuse/OTel 测试替身证明事件分发，不以 `/metrics` 单独作为通过证据。
- [ ] `SP838-T6` Owner: coordinator. Done when: gate lane 执行完毕——被 gate 模块默认不编译，README、`docs/README.md`、相关当前能力 docs（只扫现存 tracked docs，不追历史/归档计划）全文标注 experimental 或删除可用性示例，docs.rs feature 列表同步；对应 config schema/env/example 要么 gated，要么对禁用 feature 返回显式 validation error；若影响 `litellm_rs::core::<module>` import，完成 semver、CHANGELOG、deprecation/迁移说明. Verify: `cargo check`（默认 gateway 用户路径）、`cargo check --no-default-features --features "sqlite,metrics,tracing"`、`cargo check --no-default-features --features "metrics,tracing"` 与 `cargo check --all-features` 均通过；`git ls-files README.md CLAUDE.md 'docs/**/*.md' | xargs rg -n "MCP Gateway|A2A Protocol|A2A Gateway|A2AGateway|Model Context Protocol|litellm_rs::core::(mcp|a2a)"` 输出与处置一致且允许相关 protocol docs 被删除；`git ls-files config src/config docs/README.md 'docs/protocols/**/*.md' | xargs rg -n "semantic_cache|advanced_analytics|audit_logging|mcp|a2a"` 仅检查 config/schema/examples 与当前能力 docs，历史 docs/plan/specs 不作为阻塞且不会因已删除 protocol 文件失败。
- [ ] `SP838-T7` Owner: coordinator. Done when: remove lane 执行完毕，`core/mod.rs`、config schema/env/example、README、CLAUDE.md、`docs/README.md`、相关当前能力 docs（只扫现存 tracked docs，不要求已删除 protocol 文件存在）同步清理；若删除 public module，完成 semver、CHANGELOG、deprecation/迁移说明. Verify: `cargo check --all-features`; `git ls-files README.md CLAUDE.md 'docs/**/*.md' | xargs rg -n "MCP Gateway|A2A Protocol|A2A Gateway|A2AGateway|Model Context Protocol|litellm_rs::core::(mcp|a2a)"` 输出与处置一致；`git ls-files config src/config docs/README.md 'docs/protocols/**/*.md' | xargs rg -n "semantic_cache|advanced_analytics|audit_logging|mcp|a2a"` 仅检查 config/schema/examples 与当前能力 docs，无无效 no-op knobs且不会因已删除 protocol 文件失败；public import 破坏有发布记录。
- [ ] `SP838-T8` Owner: verification owner. Done when: 全量回归通过，README/CLAUDE.md/`docs/` 能力表与实际可达能力一致，public API 变更记录完整. Verify: `cargo fmt --all -- --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-features`.

## 并行拆分

- SP838-T2（纯文档）与 SP838-T4（守护检查，只动 CI/测试文件）可并行。
- SP838-T5 各子系统 PR 文件不相交可并行（W-14），但 observability+integrations 因依赖必须同组。
- SP838-T6/T7 依赖 T3 批复；T5 中安全语义子系统（guardrails/ip_access）优先。

## 验证

- [ ] `SP838-T9` Owner: verification owner. Done when: 被 wire 的每个子系统有一条本会话可复现的端到端证据记录在对应 PR body；observability+integrations 必须证明 request lifecycle event dispatch（`on_llm_start` 与 `on_llm_end`/`on_llm_error`），不是只有 `/metrics` 输出. Verify: PR body 中的命令输出（W-16：本会话证据）。

## Handoff Notes

- 与 #837 的边界：本 issue 只处理 core 子系统层；provider 目录归 #837。两者的守护检查可共享豁免清单机制但分开断言。
- batch 半接线态是最优先消除项：现状「路由存在但持久化被绕过」比完全未接线更误导；若 wire `BatchProcessor`，每个 batch item 必须调用既有 provider execution path 或保留 upstream proxy 语义，不能返回 mock/fabricated 结果。
- guardrails 若 wire，注意其 `check_output` 每次调用重新编译正则（`src/core/guardrails/prompt_injection.rs:294-303`），接线前先改为预编译（`LazyLock`），否则把性能问题带上热路径。
- `virtual_keys` 不是 stub-only：已有迁移、manager 与 SeaORM CRUD，后续处置应围绕 gateway/API 接线或 public API gate，而不是按空壳删除。
- `webhooks` 不是 stub-only：已有 delivery processor / signing / outbound POST，处置应围绕 gateway event path 接线或 gate，而不是因未挂载直接删除。
- `audit` / `enterprise.audit_logging` 必须进入矩阵和 no-op knob 检查；配置示例存在但 runtime 不执行时属于 U-26 缺口。
- remove/gate lane 可能破坏 `src/lib.rs` 暴露的 `pub mod core` import；合入前必须保留 human gate，确认 semver/CHANGELOG/deprecation/迁移说明。
