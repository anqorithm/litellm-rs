# Task Plan

## Linked Issue

GH-836 / #836

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## 实现任务

- [ ] `SP836-T1` Owner: coordinator. Done when: `specs/GH836/` 三件套通过 SpecRail packet validation. Verify: `SPEC_RAIL=/path/to/specrail; python3 "$SPEC_RAIL/checks/check_workflow.py" --repo "$SPEC_RAIL" --spec-dir "$PWD/specs/GH836"`.
- [ ] `SP836-T2` Owner: coordinator. Done when: rate limit config adds `redis_failure_mode` with default `fail_closed`, parse/merge/validation tests. Verify: `cargo test config --lib --all-features redis_failure_mode`.
- [ ] `SP836-T3` Owner: coordinator. Done when: Redis `check` failure follows policy: default fail-closed, explicit fail-open-local uses local limiter. Verify: focused limiter tests with fake Redis failure.
- [ ] `SP836-T4` Owner: coordinator. Done when: Redis `check_and_record` failure follows policy and does not create local reservation in fail-closed mode. Verify: focused limiter tests.
- [ ] `SP836-T5` Owner: coordinator. Done when: degraded metric/log helper covers check, check_and_record, and release; release failure remains non-fatal. Verify: metrics/log tests or observable counter assertions.
- [ ] `SP836-T6` Owner: docs owner. Done when: CHANGELOG/docs mention default behavior tightening and `fail_open_local` escape hatch. Verify: `rg -n "redis_failure_mode|fail_open_local|rate_limiter_degraded" docs README.md CHANGELOG.md src`.
- [ ] `SP836-T7` Owner: verification owner. Done when: format/lint/full tests pass. Verify: `cargo fmt --all -- --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-features`.

## 并行拆分

- Config and limiter code are coupled; keep in one PR to avoid a config that does nothing.

## 验证

- [ ] `SP836-T8` Owner: verification owner. Done when: PR body includes tests proving Redis failure no longer silently falls back by default. Verify: fresh command output.

## Handoff Notes

- Do not reuse storage `allow_degraded`; rate-limit semantics need explicit naming.
- Do not emit only warn logs; Redis distributed limiter failure is error-level degraded state.
- Noop Redis pool remains local limiter path and should not count as degraded.
