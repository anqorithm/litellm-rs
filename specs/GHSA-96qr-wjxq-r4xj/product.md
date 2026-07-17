# Product Spec

## Linked Issue

Private advisory: `GHSA-96qr-wjxq-r4xj`

complexity: small

## 用户问题

公开兼容入口 `config::validation::validate_url_against_ssrf` 仍维护一套旧的地址分类和 DNS
校验逻辑，而运行时 provider endpoint 已使用统一 `core::net` SSRF guard。两套规则已经发生漂移：
旧入口会放行统一策略拒绝的部分 multicast、benchmark、documentation 和其他特殊用途地址；其成功路径
测试还依赖真实公网 DNS，导致全量测试出现非确定性失败。

## 目标

- 兼容入口与统一 `public_only` SSRF 策略使用同一地址分类结果。
- 保留现有公开函数签名、HTTP/HTTPS 限制和带调用方 context 的错误信息。
- 对不可解析主机和安全策略失败继续 fail closed，不引入 fallback。
- 用不依赖公网 DNS 的确定性回归测试覆盖已确认的策略分叉。

## 非目标

- 不改变 `ProviderEndpointAccess::PrivateNetwork` 的授权语义。
- 不改变运行时连接、redirect、proxy 或 resolver 实现。
- 不新增配置字段、公开 API 或第三方依赖。
- 不扩大到非 provider endpoint 的 URL 消费方。

## Behavior Invariants

1. B-001 对任意可由兼容入口接受的 HTTP/HTTPS URL，其 host 地址是否允许必须与统一
   `public_only` provider endpoint 策略一致；兼容入口不得保留独立地址分类表。
2. B-002 兼容入口必须拒绝 multicast、benchmark、documentation、metadata、loopback、private、
   link-local、unspecified、CGNAT、reserved 以及被编码为上述地址的 host；策略拒绝不得表现为成功。
3. B-003 URL 解析失败、host 缺失、scheme 非 HTTP/HTTPS、DNS 失败或 DNS 返回任一不允许地址时必须
   返回错误；禁止 warning-only、空结果放行或旧逻辑 fallback。
4. B-004 所有错误必须保留调用方传入的 `context`，使配置错误可以定位到原字段或 endpoint。
5. B-005 现有公开函数名、参数和 `Result<(), String>` 返回契约保持不变；HTTP 与 HTTPS 的合法公网
   literal URL（含端口、路径和 query）继续成功。
6. B-006 兼容入口的单元测试不得依赖公网 DNS；策略分叉负例和合法公网正例必须使用确定性输入，且
   重复运行不得出现偶发结果。

## 验收标准

- [ ] 兼容入口委托统一 `public_only` guard，不再定义重复 IP 分类函数。
- [ ] 回归测试覆盖 `224.0.0.1`、`198.18.0.1`、`ff02::1`、`2001:db8::1` 并全部拒绝。
- [ ] 合法 HTTP/HTTPS 端口、路径、query 正例不访问公网 DNS。
- [ ] 解析、scheme、安全策略与解析失败错误均包含传入的 `context`。
- [ ] focused tests、格式、全 feature check、strict Clippy、全 feature tests 与安全审计产生新鲜证据。

## 边界情况清单

| 类别 | 判定（covered: B-xxx / N/A + 原因） |
| --- | --- |
| 空/缺失输入 | covered: B-003；空 URL、无 host 和空解析结果均失败。 |
| 错误与失败路径 | covered: B-002, B-003, B-004；分类、解析和 DNS 失败均可定位且 fail closed。 |
| 授权/权限 | N/A：该入口固定使用 `public_only`，不读取或授予私网权限。 |
| 并发/竞态 | N/A：同步、无共享可变状态；运行时 DNS rebinding 由既有统一 client 策略负责。 |
| 重试/幂等 | covered: B-006；相同确定性输入重复验证应得到相同结果。 |
| 非法状态转换 | N/A：该函数无持久状态或状态机。 |
| 兼容/迁移 | covered: B-005；公开签名与合法 HTTP/HTTPS 使用方式不变。 |
| 降级/回退 | covered: B-001, B-003；禁止回到独立分类或解析失败放行。 |
| 证据与审计完整性 | covered: B-002, B-006；已确认分叉必须有明确负例，正例不得借助公网 DNS。 |
| 取消/中断 | N/A：同步单次校验无可恢复的部分完成状态。 |

## 发布说明

内部兼容性安全修复。旧 SSRF 校验入口现在与统一 provider endpoint 策略一致，并消除了相关单元测试
对公网 DNS 的依赖；公开函数签名不变。
