# Tech Spec

## Linked Issue

GH-727 / #727

## Product Spec

Link to `product.md`.

## Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| Thinking module | `src/core/providers/thinking/mod.rs` | Uses `#[cfg(test)] mod tests;`. | Rust can resolve either `tests.rs` or `tests/mod.rs`, so a directory split preserves module identity. |
| Current test file | `src/core/providers/thinking/tests.rs` | 1100 lines grouped by provider comments. | Clear mechanical split point with no runtime code changes. |
| Provider-specific tests | NoSupport, OpenAI, Anthropic, DeepSeek, Gemini, OpenRouter, trait defaults | Each section is self-contained except shared imports. | Suitable for child modules using `use super::*;`. |

## 设计方案

1. Replace `src/core/providers/thinking/tests.rs` with `src/core/providers/thinking/tests/mod.rs`.
2. Keep the shared imports in `tests/mod.rs`.
3. Create child modules:
   - `no_support.rs`
   - `openai.rs`
   - `anthropic.rs`
   - `deepseek.rs`
   - `gemini.rs`
   - `openrouter.rs`
   - `trait_defaults.rs`
4. Add `use super::*;` to each child module so the moved tests keep the same imports.
5. Do not edit production thinking provider code.

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P1 | `tests/mod.rs` child module declarations | `cargo test core::providers::thinking --lib` discovers all thinking tests. |
| P2 | moved test chunks | Focused test count remains 75 and passes. |
| P3 | file size | `wc -l src/core/providers/thinking/tests/*.rs` shows max file below 800. |
| P4 | no runtime behavior change | `git diff --stat` and focused tests show only test layout movement. |

## 风险

- Module path changes may affect external test filters, but no public API changes.
- Mechanical moves can drop an import; focused test compilation catches this.
- This does not eliminate the full #727 backlog; the issue remains a tracker after this tranche.

## 测试计划

- [ ] SpecRail packet validation.
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo test core::providers::thinking --lib`
- [ ] `cargo check --all-features --locked`
- [ ] Line-count proof for `src/core/providers/thinking/tests/*.rs`

## 回滚方案

Revert the test module split and `specs/GH727`. No migrations or runtime config changes are involved.
