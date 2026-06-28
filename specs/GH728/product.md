# Product Spec

## Linked Issue

GH-728 / #728

## 用户问题

provider 支持状态现在分散在多条路径：HTTP gateway/router 看
`ProviderCapability`，SDK routing 用本地 `matches!` 判断，`completion()` 又维护一组
provider prefix 和动态 provider 分支。结果是同一个 provider 在不同入口的状态不一致：
例如 SDK 把 Google 选为 chat-capable，但实际执行只返回 "not implemented"。

## 目标

- 建立一个 registry 侧的 route-surface support matrix，覆盖 HTTP、SDK 和
  `completion()` 的主要入口。
- SDK chat/stream/embeddings routing 使用同一套 matrix，未实现 provider 必须稳定返回
  `NotSupported`。
- `completion()` 对 matrix 中明确 unsupported 的 provider prefix 返回明确错误，而不是模糊的
  "No suitable provider found"。
- README 提供一张面向维护者的跨入口支持矩阵。

## 非目标

- 不实现 Google/Gemini SDK chat adapter。
- 不实现所有缺失的 SDK provider。
- 不改变 provider factory、auth、HTTP payload 结构或 core provider 构造逻辑。
- 不把完整 SpecRail workflow vendoring 到本 repo。

## Behavior Invariants

1. SDK provider selection 只能选择 matrix 标记为当前 build 可用的 SDK surface。
2. SDK Google/Gemini chat 在 adapter 实现前必须返回 `SDKError::NotSupported`。
3. SDK default provider 不能绕过 surface support 判断。
4. Embeddings 仍只支持 OpenAI、Azure 和 SDK Custom，并继续保留现有 model/base_url 校验。
5. `completion()` 已知 unsupported provider prefix 必须返回明确 bad request。
6. HTTP provider capability dispatch 继续由 #729 的 `ProviderCapability` predicate 负责；matrix 不替代 execution-time capability checks。

## 验收标准

- [ ] 有一个代码级 canonical support matrix，并从 registry module 导出。
- [ ] README 中有一张跨 HTTP/SDK/completion 的 provider support matrix。
- [ ] SDK chat routing 不再选择 Google/Gemini。
- [ ] SDK direct execution 对 Google chat 返回 `NotSupported`。
- [ ] `completion()` 对 Google/Gemini prefix 给出明确 unsupported 错误。
- [ ] 通过 SpecRail packet validation、`cargo fmt --all -- --check`、focused tests、`cargo check --all-features --locked`。

## 边界情况

- Tier 1 catalog providers默认只承诺 HTTP chat/stream；只有已接入 default
  `completion()` dynamic route 的 provider 才在 matrix 中标记 completion 支持。
- Feature-gated HTTP provider surface 必须按当前 cargo feature 判断是否可选。
- `api_base` 的 generic OpenAI-compatible 用法不能被未列入 matrix 的 provider-like model prefix 误拦截。

## 发布说明

这是 provider support contract 收敛。Google/Gemini SDK chat 的行为会从运行期
`ProviderError("not implemented")` 收敛为稳定的 `NotSupported`，并在 README 中明确当前支持边界。
