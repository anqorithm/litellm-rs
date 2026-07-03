# Task Plan

## Linked Issue

GH-832 / #832

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## 实现任务

- [ ] `SP832-T1` Owner: coordinator. Done when: `specs/GH832/` 三件套通过 SpecRail packet validation. Verify: `SPEC_RAIL=/path/to/specrail; python3 "$SPEC_RAIL/checks/check_workflow.py" --repo "$SPEC_RAIL" --spec-dir "$PWD/specs/GH832"`.
- [ ] `SP832-T2` Owner: coordinator. Done when: CORS middleware 在 Actix 执行顺序中位于 auth/rate-limit 外层，注释解释 wrap 逆序执行. Verify: focused test fails before reorder and passes after; `git diff -- src/server/http.rs`.
- [ ] `SP832-T3` Owner: coordinator. Done when: 标准 CORS preflight 不触发 missing-auth 401；若需要 helper，则 helper 严格匹配 `OPTIONS` + `Origin` + `Access-Control-Request-Method`. Verify: `cargo test server::http cors --all-features` 或对应集成测试。
- [ ] `SP832-T4` Owner: coordinator. Done when: 非 preflight 请求鉴权不变，未授权 POST 仍 401 且带 CORS header. Verify: focused actix test for unauthenticated POST with allowed Origin.
- [ ] `SP832-T5` Owner: verification owner. Done when: 格式、lint、全量测试通过. Verify: `cargo fmt --all -- --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-features`.

## 并行拆分

- 单文件/小测试修复，不需要并行拆分。

## 验证

- [ ] `SP832-T6` Owner: verification owner. Done when: 手工或集成测试记录 preflight response status 与 `Access-Control-Allow-*` 头. Verify: PR body 附测试输出。

## Handoff Notes

- 不要把 `/v1/*` 加到 public route。
- 不要无条件允许所有 `OPTIONS`；只处理标准 CORS preflight。
- 若调整 middleware 顺序，确认 auth/rate-limit 相对顺序不被破坏。
