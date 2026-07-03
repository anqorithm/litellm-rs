# Task Plan

## Linked Issue

GH-835 / #835

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## 实现任务

- [ ] `SP835-T1` Owner: coordinator. Done when: `specs/GH835/` 三件套通过 SpecRail packet validation. Verify: `SPEC_RAIL=/path/to/specrail; python3 "$SPEC_RAIL/checks/check_workflow.py" --repo "$SPEC_RAIL" --spec-dir "$PWD/specs/GH835"`.
- [ ] `SP835-T2` Owner: coordinator. Done when: batch missing-provider helper no longer returns `GatewayError::Config`; it returns a semantic 4xx error. Verify: focused test for `/v1/batches` no provider.
- [ ] `SP835-T3` Owner: coordinator. Done when: image proxy no-provider and unsupported requested model errors return semantic 4xx. Verify: focused tests for image edits/variations provider absent and model unsupported.
- [ ] `SP835-T4` Owner: coordinator. Done when: true internal config errors still map to 500. Verify: renderer regression test with `GatewayError::Config`.
- [ ] `SP835-T5` Owner: verification owner. Done when: old tests asserting 500 are updated, not removed, and all focused tests pass. Verify: `cargo test server::routes::ai --all-features`.
- [ ] `SP835-T6` Owner: verification owner. Done when: format/lint/full tests pass. Verify: `cargo fmt --all -- --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-features`.

## 并行拆分

- Batch and image helpers are small but share renderer semantics; keep in one PR.

## 验证

- [ ] `SP835-T7` Owner: verification owner. Done when: PR body includes status/body snippets or focused test output proving 4xx and non-internal code. Verify: fresh command output.

## Handoff Notes

- Do not globally remap `GatewayError::Config`.
- Do not weaken upstream proxy error behavior.
- Keep #839 boundary: this issue fixes concrete route-local status errors only.
