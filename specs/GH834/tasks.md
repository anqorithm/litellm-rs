# Task Plan

## Linked Issue

GH-834 / #834

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## 实现任务

- [ ] `SP834-T1` Owner: coordinator. Done when: `specs/GH834/` 三件套通过 SpecRail packet validation. Verify: `SPEC_RAIL=/path/to/specrail; python3 "$SPEC_RAIL/checks/check_workflow.py" --repo "$SPEC_RAIL" --spec-dir "$PWD/specs/GH834"`.
- [ ] `SP834-T2` Owner: coordinator. Done when: image generation route resolves explicit/effective authz model before provider invocation. Verify: focused unit tests for resolver.
- [ ] `SP834-T3` Owner: coordinator. Done when: missing model + restricted API key cannot reach provider unless a unique allowed effective model is resolved. Verify: route test with sentinel provider/router not called on deny.
- [ ] `SP834-T4` Owner: coordinator. Done when: unrestricted key/no allowed_models preserves old omitted-model behavior. Verify: route or resolver test.
- [ ] `SP834-T5` Owner: coordinator. Done when: explicit model allowed/denied behavior and image edit/variation model-required behavior remain unchanged. Verify: focused regression tests.
- [ ] `SP834-T6` Owner: verification owner. Done when: format/lint/tests pass. Verify: `cargo fmt --all -- --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-features`.

## 并行拆分

- Single security fix PR; do not split authz and tests.

## 验证

- [ ] `SP834-T7` Owner: verification owner. Done when: PR body includes before/after evidence that missing-model restricted request is denied before provider. Verify: focused test output from this session.

## Handoff Notes

- Do not duplicate allowed_models matching logic; call the existing helper.
- Do not rely on provider-side defaults for restricted keys unless gateway can prove the same effective model before provider call.
- Treat ambiguous default resolution as deny/4xx, not as allow.
