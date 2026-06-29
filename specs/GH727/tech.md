# Tech Spec

## Linked Issue

GH-727 / #727

## Product Spec

Link to `product.md`.

## Codebase Context

| Area | Files | Current behavior | Why relevant |
| --- | --- | --- | --- |
| Azure Assistants module | `src/core/providers/azure/assistants.rs` | Contains runtime code plus a large inline `#[cfg(test)] mod tests`. | Inline tests make the file exceed the U-16 ceiling. |
| Existing Azure pattern | `src/core/providers/azure/chat.rs`, `src/core/providers/azure/chat_tests.rs` | `chat.rs` uses `#[path = "chat_tests.rs"] mod tests;`. | Same pattern preserves test module identity while keeping the test file next to the module. |
| Assistants tests | `src/core/providers/azure/assistants.rs` inline tests | 52 test functions covering serialization, validation, errors, URL builders, clone/debug. | Suitable for a mechanical move to `assistants_tests.rs` using `use super::*;`. |

## 设计方案

1. Move the body of `#[cfg(test)] mod tests` from `assistants.rs` into `assistants_tests.rs`.
2. Replace the inline module with `#[cfg(test)] #[path = "assistants_tests.rs"] mod tests;`.
3. Keep `use super::*;` and the existing `serde_json::json` import in the moved file.
4. Do not edit production Azure Assistants code.

## Product-to-Test Mapping

| Product invariant | Implementation area | Verification |
| --- | --- | --- |
| P1 | `assistants.rs` test module declaration | `cargo test core::providers::azure::assistants --lib --all-features` discovers all moved tests. |
| P2 | moved test module body | Focused Azure Assistants tests pass with unchanged assertions. |
| P3 | file size | `wc -l src/core/providers/azure/assistants.rs src/core/providers/azure/assistants_tests.rs` shows both files below 800. |
| P4 | no runtime behavior change | `git diff --stat` and focused tests show only test layout movement plus SpecRail docs. |

## 风险

- Module path remains `assistants::tests` because `assistants.rs` still declares `mod tests`; only the file backing the module changes.
- Mechanical moves can drop an import or break access to private helper methods; focused test compilation catches this.
- This does not eliminate the full #727 backlog; the issue remains a tracker after this tranche.

## 测试计划

- [ ] SpecRail packet validation.
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo test core::providers::azure::assistants --lib --all-features`
- [ ] `cargo check --all-features --locked`
- [ ] Line-count proof for `src/core/providers/azure/assistants.rs` and `assistants_tests.rs`

## 回滚方案

Revert the Azure Assistants test module split and `specs/GH727` edits. No migrations or runtime config changes are involved.
