# Task Plan

## Linked Issue

GH-842 / #842

## Spec Packet

- Product: `product.md`
- Tech: `tech.md`

## 实现任务

- [ ] `SP842-T1` Owner: coordinator. Done when: `specs/GH842/` 三件套通过 SpecRail packet validation. Verify: `SPEC_RAIL=/path/to/specrail; python3 "$SPEC_RAIL/checks/check_workflow.py" --repo "$SPEC_RAIL" --spec-dir "$PWD/specs/GH842"`.
- [ ] `SP842-T2` Owner: maintainer. Done when: #842 批复 `Arc<ChatCompletionRequest>` / `Arc<RequestContext>` / `Arc<KeyManager>` 方向和 benchmark 口径（SpecRail human gate `spec_approval`）. Verify: #842 issue thread 明确批复。
- [ ] `SP842-T3` Owner: coordinator. Done when: chat non-stream request flow 改为共享原始 request，`response_cache`、budget estimation、token policy 不再各持一份完整 clone；行为测试覆盖 cache hit、预算不足、provider 成功、provider 失败. Verify: `cargo test server::routes::ai::chat --lib --all-features`。
- [ ] `SP842-T4` Owner: coordinator. Done when: chat stream request flow 改为共享原始 request，内部 usage 注入只发生在 provider request builder 中，客户端 usage 输出过滤行为不变. Verify: stream 聚焦测试覆盖 `client_requested_usage=true/false`。
- [ ] `SP842-T5` Owner: coordinator. Done when: `RequestContext` 在 middleware extensions 与 handlers 中以共享 handle 传递，鉴权 metadata 保持可见，敏感 headers 继续排除. Verify: `cargo test server::middleware::auth --lib --all-features`; `cargo test server::routes::ai::context --lib --all-features`。
- [ ] `SP842-T6` Owner: coordinator. Done when: `AppState.key_manager` 或 `KeyManager.hmac_secret` 共享化，AI route spend 调用不再复制 HMAC secret 字符串；key management route 行为不变. Verify: `cargo test core::keys --lib --all-features`; key route 聚焦测试。
- [ ] `SP842-T7` Owner: verification owner. Done when: 大 payload allocation test 或 benchmark 证明 request/context/key_manager clone 分配下降，PR body 附命令与前后数据. Verify: 记录实际 bench/测试输出。
- [ ] `SP842-T8` Owner: verification owner. Done when: 全仓确定性验证通过. Verify: `cargo test --all-features`。

## 并行拆分

- SP842-T3/T4 都修改 `src/server/routes/ai/chat.rs`，不得并行写同文件。
- SP842-T5 会触碰 shared handler signatures，需在 SP842-T3/T4 前后由同一 owner 串行收敛。
- SP842-T7 可由只读验证 lane 在实现分支稳定后并行运行。

## Handoff Notes

- 不要把本 issue 与 #840 预算编排合并；本 issue 只降低分配，不改变预算生命周期。
- 任何静态 guard 都要允许最终 provider request 的必要 clone，不能用过宽规则阻止合法拥有权转换。
