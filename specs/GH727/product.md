# Product Spec

## Linked Issue

GH-727 / #727

## 用户问题

当前 main 仍有大量 Rust 文件超过 U-16 的 800 行硬上限。一次性拆完会形成不可 review 的大 PR；
#727 要求用小 PR tranche 逐步拆分，并且每个 PR 只拥有一个文件或紧密文件家族。

## 本 tranche 目标

- 拆分 `src/core/providers/thinking/tests.rs`，它当前 1100 行，是 #727 当前 top offenders 之一。
- 按 provider group 将测试移动到独立模块，保持测试语义和 public API 不变。
- 所有新增 Rust 文件低于 800 行。

## 非目标

- 不修改 thinking provider runtime 代码。
- 不重构 OpenAI/Anthropic/DeepSeek/Gemini/OpenRouter thinking 行为。
- 不在本 PR 中处理其余 50 个大文件。
- 不关闭 #727，除非 issue owner 决定一个 tranche PR 足以满足 tracker。

## Behavior Invariants

1. 所有原有 thinking tests 仍由 `core::providers::thinking::tests::*` 测试树运行。
2. 测试移动只能改变 module path，不改变断言、fixtures 或 production code。
3. `src/core/providers/thinking/tests/*.rs` 单文件必须低于 800 行。
4. `cargo test core::providers::thinking --lib` 必须通过。

## 验收标准

- [ ] `src/core/providers/thinking/tests.rs` 被替换为 `tests/mod.rs` + provider-specific test files。
- [ ] 最大新增 test file 低于 800 行。
- [ ] Focused thinking tests 通过。
- [ ] `cargo fmt --all -- --check` 和 `cargo check --all-features --locked` 通过。
- [ ] PR body 明确该 PR 是 #727 的 first tranche，不自动关闭 tracker issue。

## 发布说明

No runtime behavior change. This is a test layout maintenance split for U-16 compliance.
