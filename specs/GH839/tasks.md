# Task Plan

## Linked Issue

GH-839 / #839

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## 实现任务

- [ ] `SP839-T1` Owner: coordinator. Done when: `specs/GH839/` 三件套通过 SpecRail packet validation. Verify: `SPEC_RAIL=/path/to/specrail; python3 "$SPEC_RAIL/checks/check_workflow.py" --repo "$SPEC_RAIL" --spec-dir "$PWD/specs/GH839"`.
- [ ] `SP839-T2` Owner: maintainer. Done when: #839 批复 `Cancelled` 状态码（建议 499）、管理端状态码语义化的对外行为变化，以及 `ApiResponse::to_http_response` 保留/deprecated 或删除 breaking change 决策（SpecRail human gate `spec_approval`）. Verify: #839 issue thread 明确批复。
- [ ] `SP839-T3` Owner: coordinator. Done when: `http_facts()` canonical 映射模块合入（含当前动态 rate-limit headers、`X-RateLimit-Limit`、分离的 OpenAI vs canonical/internal code 词表、上游 OpenAI `ApiError` 字段透传策略），并拆出 feature-neutral status/code facts 供 `ProviderError::http_status` / `ContextualError::http_status` 和 lite/no-default build 使用；gateway-only adapter 才使用 actix status/header 类型；match 无 `_` 分支，全变体单测覆盖. Verify: `cargo test utils::error --lib --all-features`; `cargo check --no-default-features --features lite`; `rg -n "fn http_facts|ErrorHttpFacts|http_status" src/utils/error src/core/providers`.
- [ ] `SP839-T4` Owner: coordinator. Done when: 一致性遍历测试合入并以现状两张表为对照暴露全部漂移（首个红灯清单进 PR body）. Verify: 测试先红后绿的记录；`Cancelled` 漂移在清单中。
- [ ] `SP839-T5` Owner: coordinator. Done when: `openai_errors.rs`、`gateway_error/response.rs`、`ProviderError::http_status` / `ContextualError::http_status`、`chat_sse.rs` 和 `responses_stream.rs` 的 stream error classifier 改为消费同一 `http_facts` / code facts，两张手写 match 表和 stream 独立分类表删除；OpenAI `error.code` 与 canonical/internal `code`/`canonical_code` 响应值保持现状. Verify: `rg -c "StatusCode::" src/server/routes/ai/openai_errors.rs src/utils/error/gateway_error/response.rs src/core/providers/unified_provider_methods.rs` 显著收敛；一致性测试、stream code 词表回归测试绿色。
- [ ] `SP839-T6` Owner: coordinator. Done when: OpenAI-compatible AI 路由统一显式渲染，`JsonConfig`/`QueryConfig`/`PathConfig` error_handler、auth/rate-limit 中间件拒绝、direct `openai_errors::*` helper、本地 multipart/validation/auth 失败、raw upstream proxy 非 2xx（batches/images/Gemini proxy）接入同一 OpenAI renderer，错误 body `request_id` 与头一致；管理端 `/v1/*` 与 `/v1/pricing` 路由不被转换为 OpenAI 形状. Verify: 集成测试覆盖 extractor 三类失败、bad key、rate-limit、direct validation/helper 失败、handler 失败、raw proxy upstream 500 形状一致；`rg -n "request_id: None|openai_errors::(validation_error|unauthorized_error)" src` 仅剩明确带 ctx 的调用或兜底注释。
- [ ] `SP839-T7` Owner: coordinator. Done when: `src/server/routes/mod.rs` 的管理端 errors helpers 携带语义状态码，`/v1/pricing` 保持非 OpenAI route-family 错误形状，public auth recovery / verification 反枚举 flow 明确豁免或测试证明响应不泄露账户/token 有效性，`ApiResponse::to_http_response` 按 SP839-T2 决策保留/deprecated 或以 breaking change 明示删除，信封字段兼容测试通过. Verify: 若保留则编译测试覆盖 `to_http_response`; 若删除则 CHANGELOG/PR body 标明 public API break; `cargo test server::routes --all-features`; pricing/auth recovery focused tests。
- [ ] `SP839-T8` Owner: verification owner. Done when: 全量回归 + CHANGELOG 状态码变化矩阵完成. Verify: `cargo fmt --all -- --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-features`.

## 并行拆分

- SP839-T3+T4 一个 PR（表 + 测试，先红后绿）；SP839-T5 第二个 PR；SP839-T6 与 SP839-T7 文件不相交可并行两个 PR（W-14：`src/server/routes/ai/`+middleware/extractor config vs `src/server/routes/mod.rs`+管理端路由）。
- 全链依赖 SP839-T2 的两个对外行为批复。

## 验证

- [ ] `SP839-T9` Owner: verification owner. Done when: 手工对照记录（非法 JSON / 非法 query/path / 错误 key / rate-limit / 未知模型 / 管理端 404）六类请求的 status + body 形状 + request_id + rate-limit headers 进入收尾 PR body. Verify: PR body 中的 `curl -i` 输出（W-16 本会话证据）。

## Handoff Notes

- 与 #833（Retry-After）的接口：`ErrorHttpFacts.headers` 是 #833 的填充点；本 issue 不补全缺失策略，但必须保留当前 provider/gateway 已经输出的 `Retry-After` 与 `X-RateLimit-*`。
- 与 #715 的关系：#715 已把 retry policy 从错误变体拆出，本 issue 不回退该设计，只收敛「变体 → HTTP」的最后一跳。
- 流式（SSE）中途失败的错误事件形状本 spec 只约束错误码来源（同一张表），事件结构不动。
