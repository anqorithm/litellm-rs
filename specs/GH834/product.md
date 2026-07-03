# Product Spec

## Linked Issue

GH-834 / #834

## 用户问题

`/v1/images/generations` 只有在请求显式包含 `model` 时才调用
`enforce_api_key_model_and_token_limits`。当 `model` 省略时，检查整段跳过，后续 provider
使用默认模型执行。带 `allowed_models` 限制的 API key 可以借此绕过模型白名单。

这是 security issue：用户授权范围由 API key policy 定义，默认模型不能成为绕过路径。

## 目标

- image generation 请求无论是否显式传入 `model`，都必须对实际生效模型执行 API key model 限制。
- 无法在 provider 调用前确定唯一生效模型时，受限 key 请求必须 fail-closed，而不是交给 provider 默认。
- 不影响 image edit / variation proxy 已要求 multipart `model` 的路径。

## 非目标

- 不改变 provider 默认模型本身。
- 不扩大 allowed_models 语义到非模型资源。
- 不重构整个 image routing 架构。

## Behavior Invariants

1. `allowed_models` 存在且请求省略 `model` 时，gateway 不得在未校验生效模型的情况下调用 provider。
2. 若能根据配置解析唯一 effective image generation model，则对该模型调用现有
   `enforce_api_key_model_and_token_limits`。
3. 若无法唯一解析 effective model，受限 key 请求返回 OpenAI 形状 4xx，要求显式 `model` 或说明模型不被允许。
4. 没有 model 限制的 key 保持现有默认模型行为。
5. 错误不能泄露无关 provider secrets 或内部 routing config。

## 验收标准

- [ ] 测试：`allowed_models=["gpt-4o"]`，image generation 省略 `model`，provider 默认 image model 不在白名单时返回 4xx，provider 未调用。
- [ ] 测试：省略 `model` 但可解析默认模型且在白名单内时请求通过 authz 检查。
- [ ] 测试：显式 `model` 的既有允许/拒绝行为不回退。
- [ ] image edit / variation proxy 仍要求 multipart model，并继续先 authz 再 routing。

## 边界情况

- 多个 image provider 候选且默认模型不唯一：受限 key fail-closed。
- allowed_models 为空/未设置：不新增强制 model 要求。
- wildcard / pattern 规则若已存在，必须复用现有 `enforce_api_key_model_and_token_limits` 语义。

## 发布说明

修复 image generation 省略 `model` 时绕过 API key allowed_models 的安全问题。受限 key 可能需要显式传入允许的 image model。
