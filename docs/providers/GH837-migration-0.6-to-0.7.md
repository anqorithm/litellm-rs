# GH837 provider 0.6 → 0.7 迁移

## `amazon_nova`

### 时间线

- 0.6.x：`providers-extended` 下的 native `amazon_nova` module 已标记
  deprecated，但 `AmazonNovaConfig`、`AmazonNovaErrorMapper`、
  `AmazonNovaModel`、`AmazonNovaModelRegistry` 与 `AmazonNovaProvider`
  的公开 symbol、构造签名和既有运行时行为保持不变。
- 0.7.0：在 version workflow 与 public compatibility gate 通过后，移除
  duplicate native module，保留 `amazon_nova` catalog route。这是公开 Rust
  API 的 breaking change，不是运行时 provider 降级。

### Catalog 等价策略

Catalog route 保留 native 的固定 endpoint
`https://api.nova.amazon.com/v1`、Bearer auth 与
`AMAZON_NOVA_API_KEY` contract，并只声明 native provider 已实现的 chat、
streaming chat 与 tool-calling capabilities。

Catalog policy 是模型、能力、价格与 alias 的唯一权威；保留的 native registry
仅是 0.6 compatibility projection。Catalog 保留五个 canonical model：Nova 2
Lite、Pro、Lite、Micro 与 Premier，以及各自的 context/output limits、token
pricing、tool 和 multimodal metadata。缺少其中任一项都不满足后续 0.7.0
demotion gate。

### 迁移

通过 gateway/provider configuration 使用 `amazon_nova` selector 的调用方无需
更改 selector。直接 import native Rust types 的下游 crate 应在 0.7.0 前迁移到
registry/catalog construction path；不要复制 native module 或依赖其内部 macro
实现。

## `github`

### 时间线

- 0.6.x：`providers-extended` 下的 native `github` module 已标记 deprecated，
  但 `GitHubConfig`、`GitHubError`、`GitHubModel`、`GitHubProvider` 与
  `get_available_models` / `get_model_info` 的公开 symbol、构造签名和既有运行时
  行为保持不变。
- 0.7.0：在 version workflow 与 public compatibility gate 通过后，移除 duplicate
  native module，保留 `github` catalog route（`github_copilot` 是不同 scope，
  不受影响）。这是公开 Rust API 的 breaking change，不是运行时 provider 降级。

### Catalog 等价策略

Catalog route 保留 native 的固定 endpoint `GITHUB_MODELS_API_BASE`
（`https://models.inference.ai.azure.com`）、Bearer auth 与 `GITHUB_TOKEN`
contract，并只声明 native provider 已实现的 chat、streaming chat 与 tool-calling
capabilities。Health 由 OpenAI-compatible catalog route 的标准 `/models` 探针
提供，没有 github 专属 health 机制。

Catalog policy（`registry::github_policy`）是模型、能力与价格的唯一权威；保留的
native registry 仅是 0.6 compatibility projection。Catalog 保留全部 16 个 canonical
model（OpenAI gpt-4o/gpt-4o-mini/o1-preview/o1-mini、Meta Llama 3.1 405B/70B/8B、
Mistral Large/Small、Cohere Command R+/R、AI21 Jamba 1.5 Large/Mini、Phi 3.5
MoE/Mini/Vision），以及各自的 context/output limits、token pricing 与 tool /
multimodal metadata。缺少其中任一项都不满足后续 0.7.0 demotion gate。

### 迁移

通过 gateway/provider configuration 使用 `github` selector 的调用方无需更改
selector。直接 import native Rust types 的下游 crate 应在 0.7.0 前迁移到
registry/catalog construction path；不要复制 native module。

## `custom_api`

### 时间线

- 0.6.x：`providers-extended` 下的 `custom_api` module、`CustomHttpxConfig`、
  `CustomApiErrorMapper` 与 `CustomHttpxProvider` 已标记 deprecated。公开
  symbol、构造签名与既有运行时行为保持不变。
- 0.7.0：在 version workflow 与 public compatibility gate 通过后，删除该公开
  module 和 native implementation。这是明确的 breaking change。

### 产品边界

把任意 URL、任意 HTTP method、自由格式 request template 或任意 response parser
组合成 provider，不再是 LiteLLM-RS 的产品目标。0.6.x 的 deprecation 不扩展、
修补或承诺这些通用适配能力，也不改变现有 dispatch；它只提供迁移窗口。

### 替代方案

1. 对 OpenAI-compatible endpoint，使用 registry/catalog 支持的 provider；
   若目标尚未收录，提交带固定 endpoint、认证环境变量、模型与 capability metadata
   的 catalog definition。
2. 对存在专有协议、认证、streaming 或错误语义的服务，使用 dedicated typed
   provider integration，并接入正式 registry/factory/dispatch 路径。
3. 对应用私有的任意 HTTP 编排，把 URL/method/template/parser 逻辑留在应用侧，
   在进入 gateway 前转换为受支持的 provider contract。

不要依赖恢复 `custom_api` 目录、复制其 macro-generated provider，或把动态任意
endpoint 静默包装为 canonical provider。0.7.0 迁移应选择上述明确拥有者之一。
