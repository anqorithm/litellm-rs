# Product Spec

## Linked Issue

GH-835 / #835

## 用户问题

客户端请求未配置的 batch/image 模型时，gateway 返回 HTTP 500。实际原因是 route-local
“provider/model 未配置”错误使用 `GatewayError::Config`，而 OpenAI error renderer 将所有
`Config` 统一映射为 500。用户可触发的 bad model / unsupported endpoint 因此污染 5xx 指标并诱导重试。

## 目标

- batch/image 的“未配置 provider/model”返回 OpenAI-compatible 4xx。
- 真正的启动/内部配置错误仍是 5xx。
- 错误 code/type 能让客户端区分 unsupported/missing model 与服务端故障。

## 非目标

- 不全局改变 `GatewayError::Config` 的 HTTP 映射。
- 不修复 #839 的全局错误映射重复问题。
- 不改变 batch/image proxy 的成功路径或 provider selection。

## Behavior Invariants

1. 没有配置任何 batch provider 时，`/v1/batches` 返回 4xx，不返回 500。
2. image edit/variation 未配置 provider 或 requested model 没有候选时返回 4xx，不返回 500。
3. pricing 数据损坏、URL/header 构造错误等内部配置/程序错误仍可返回 5xx。
4. OpenAI-compatible body shape 保持 `{error:{message,type,param,code}}`。
5. 已有 tests 中断言 500 的用例必须改为新语义，不能删除覆盖。

## 验收标准

- [ ] `/v1/batches` 无 provider 返回 400/404，error code 不是 `internal_error`。
- [ ] `/v1/images/edits` / `/v1/images/variations` 无 provider 或 model 未配置返回 400/404。
- [ ] `GatewayError::Config` 的全局 renderer 仍对真正 internal config 映射 500。
- [ ] 5xx metrics 不再因这类客户端请求增加。

## 边界情况

- provider configured but upstream returns non-2xx：不在本 issue 改语义。
- provider URL/header invalid：属于配置错误，可保持 5xx。
- missing pricing 与 unpriced model 归 #831，不在本 issue 混修。

## 发布说明

未配置 batch/image model/provider 的客户端错误现在返回 4xx，而不是 500。
