# Task Plan

## Linked Issue

GH-839 / #839

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## 实现任务

- [ ] `SP839-T1` Owner: coordinator. Done when: `specs/GH839/` 三件套通过 SpecRail packet validation. Verify: `SPEC_RAIL=/path/to/specrail; python3 "$SPEC_RAIL/checks/check_workflow.py" --repo "$SPEC_RAIL" --spec-dir "$PWD/specs/GH839"`.
- [ ] `SP839-T2` Owner: maintainer. Done when: #839 批复 `Cancelled` 状态码（建议 499）与管理端状态码语义化的对外行为变化（SpecRail human gate `spec_approval`）. Verify: #839 issue thread 明确批复。
- [ ] `SP839-T3` Owner: coordinator. Done when: `http_facts()` canonical 映射模块合入（含 headers 预留位），match 无 `_` 分支，全变体单测覆盖. Verify: `cargo test utils::error --lib --all-features`; `rg -n "fn http_facts" src/utils/error`.
- [ ] `SP839-T4` Owner: coordinator. Done when: 一致性遍历测试合入并以现状两张表为对照暴露全部漂移（首个红灯清单进 PR body）. Verify: 测试先红后绿的记录；`Cancelled` 漂移在清单中。
- [ ] `SP839-T5` Owner: coordinator. Done when: `openai_errors.rs` 与 `gateway_error/response.rs` 改为消费 `http_facts()`，两张手写 match 表删除. Verify: `rg -c "StatusCode::" src/server/routes/ai/openai_errors.rs src/utils/error/gateway_error/response.rs` 显著收敛；一致性测试绿色。
- [ ] `SP839-T6` Owner: coordinator. Done when: `/v1/*` 路由统一显式渲染 + `JsonConfig` error_handler 接入，错误 body `request_id` 与头一致. Verify: 集成测试三阶段失败形状一致；`rg -n "request_id: None" src/utils/error` 仅剩兜底注释。
- [ ] `SP839-T7` Owner: coordinator. Done when: 管理端 errors helpers 携带语义状态码，`ApiResponse::to_http_response` 删除，信封字段兼容测试通过. Verify: `rg -n "to_http_response" src` 零命中; `cargo test server::routes --all-features`。
- [ ] `SP839-T8` Owner: verification owner. Done when: 全量回归 + CHANGELOG 状态码变化矩阵完成. Verify: `cargo fmt --all -- --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-features`.

## 并行拆分

- SP839-T3+T4 一个 PR（表 + 测试，先红后绿）；SP839-T5 第二个 PR；SP839-T6 与 SP839-T7 文件不相交可并行两个 PR（W-14：`src/server/routes/ai/` vs `src/server/http.rs`+管理端路由）。
- 全链依赖 SP839-T2 的两个对外行为批复。

## 验证

- [ ] `SP839-T9` Owner: verification owner. Done when: 手工对照记录（非法 JSON / 错误 key / 未知模型 / 管理端 404）四类请求的 status + body 形状 + request_id 进入收尾 PR body. Verify: PR body 中的 `curl -i` 输出（W-16 本会话证据）。

## Handoff Notes

- 与 #833（Retry-After）的接口：`ErrorHttpFacts.headers` 是 #833 的填充点，本 issue 只保证位子存在且被两个适配器透传。
- 与 #715 的关系：#715 已把 retry policy 从错误变体拆出，本 issue 不回退该设计，只收敛「变体 → HTTP」的最后一跳。
- 流式（SSE）中途失败的错误事件形状本 spec 只约束错误码来源（同一张表），事件结构不动。
