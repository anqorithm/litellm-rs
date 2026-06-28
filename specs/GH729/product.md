# Product Spec

## Linked Issue

GH-729 / #729

## 用户问题

provider 能力契约现在有两套表述：实际 HTTP/router 路径通过 `ProviderCapability`
选择部署，但 `sub_traits.rs` 仍暴露 `LLMChat`、`LLMEmbed`、`LLMStream`
这组未接入 call site 的旧 carve-out。维护者和下游用户无法判断 optional
capabilities 应该通过 trait object、sub-trait，还是能力枚举来分发。

## 目标

- 明确 `ProviderCapability` 是当前 runtime dispatch 的唯一能力契约。
- 保留 deprecated sub-traits 作为 library compatibility adapters，但不再把它们描述成未来主架构。
- optional capabilities（streaming、embeddings、image generation）必须有稳定的 supports/error 语义。
- 用测试覆盖 chat-only provider 和 optional-capability provider 的判断行为。

## 非目标

- 不收敛 provider factory、registry 或 runtime enum。
- 不新增 provider 实现。
- 不把 `LLMProvider` 拆成新的 object-safe trait 栈。
- 不删除 public deprecated sub-traits，避免 minor release 破坏下游编译。

## Behavior Invariants

1. `LLMProvider::supports_capability(capability)` 必须只由 `capabilities()` 返回值决定。
2. `supports_streaming()`、`supports_embeddings()`、`supports_image_generation()` 必须与对应 `ProviderCapability` 保持一致。
3. 未声明 optional capability 的 provider 调用对应 optional method 时必须返回稳定的 `ProviderError::NotSupported`，不能静默 fallback。
4. deprecated `LLMChat`、`LLMEmbed`、`LLMStream` 只能作为 `LLMProvider` 的 compatibility adapters；新路由不得依赖它们。
5. router/gateway 的 capability 选择必须使用同一套 capability predicate，不能复制新的判断逻辑。

## 验收标准

- [ ] `sub_traits.rs` 明确说明 removal/migration guidance，并避免暗示它会成为 runtime dispatch path。
- [ ] `LLMProvider` 或 provider wrapper 暴露统一 capability predicate。
- [ ] optional capabilities 的 dispatch contract 在代码注释和测试中可见。
- [ ] 测试覆盖 chat-only provider 和至少一个 optional capability provider。
- [ ] 通过 `cargo fmt --all -- --check`、focused provider/capability tests、`cargo check --all-features --locked`。

## 边界情况

- 下游代码仍可能 import deprecated sub-traits；本变更不得删除这些 symbols。
- 有些 provider 支持 embeddings 但不支持 chat；capability predicate 必须允许这种组合。
- 路由层可能遇到健康不可用和能力不支持两类错误；两者不能混淆。

## 发布说明

这是 provider architecture clarification。用户可见行为应保持不变，但开发者文档和测试会明确：新增/修改 provider 时必须声明 `ProviderCapability`，router 按 capability 选择部署。
