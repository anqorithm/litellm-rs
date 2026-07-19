# `custom_api` 0.6 → 0.7 迁移

## 时间线

- 0.6.x：`providers-extended` 下的 `custom_api` module、`CustomHttpxConfig`、
  `CustomApiErrorMapper` 与 `CustomHttpxProvider` 已标记 deprecated。公开
  symbol、构造签名与既有运行时行为保持不变。
- 0.7.0：在 version workflow 与 public compatibility gate 通过后，删除该公开
  module 和 native implementation。这是明确的 breaking change。

## 产品边界

把任意 URL、任意 HTTP method、自由格式 request template 或任意 response parser
组合成 provider，不再是 LiteLLM-RS 的产品目标。0.6.x 的 deprecation 不扩展、
修补或承诺这些通用适配能力，也不改变现有 dispatch；它只提供迁移窗口。

## 替代方案

1. 对 OpenAI-compatible endpoint，使用 registry/catalog 支持的 provider；
   若目标尚未收录，提交带固定 endpoint、认证环境变量、模型与 capability metadata
   的 catalog definition。
2. 对存在专有协议、认证、streaming 或错误语义的服务，使用 dedicated typed
   provider integration，并接入正式 registry/factory/dispatch 路径。
3. 对应用私有的任意 HTTP 编排，把 URL/method/template/parser 逻辑留在应用侧，
   在进入 gateway 前转换为受支持的 provider contract。

不要依赖恢复 `custom_api` 目录、复制其 macro-generated provider，或把动态任意
endpoint 静默包装为 canonical provider。0.7.0 迁移应选择上述明确拥有者之一。
