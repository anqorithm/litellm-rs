# GH953 产品规格：认证凭证与 API-key 生命周期加固

关联 issue：`GH953`

## 目标

确保认证边界不会泄露凭证，也不会在 key 或其 owner 失效后继续接受缓存中的旧状态。

## 非目标

- 不改变 JWT、session 或 API-key 的编码与哈希算法。
- 不替换数据库、Redis 或现有 repository 抽象。
- 不调整权限、轮换策略或公开成功响应格式。

## 行为不变量

- `GH953-B1`：`AuthMethod` 的 `Debug`/日志输出只能显示认证方式，不能包含 JWT、API key 或 session ID 的任何字节。
- `GH953-B2`：有关联 owner 的 API key 仅在 owner 存在且为 active 时有效；无 owner 的系统/团队 key 保持现有语义。
- `GH953-B3`：普通验证和详细验证对 key 状态、过期时间、owner 缺失与 owner inactive 使用同一判定规则。
- `GH953-B4`：认证路径不读取 API-key 缓存快照；数据库状态变化后，旧缓存不得继续授权。
- `GH953-B5`：无效凭证返回稳定的认证失败；存储等基础设施错误使用不同的通用失败消息，客户端不获得内部错误细节。

## 验收标准

- `GH953-AC1`：回归测试证明三类凭证均被完整遮蔽。
- `GH953-AC2`：回归测试证明 inactive owner 与 missing owner 在两种验证入口中均被拒绝。
- `GH953-AC3`：回归测试证明不执行缓存失效时，数据库中已撤销的 key 仍会被两种验证入口拒绝。
- `GH953-AC4`：基础设施错误与 invalid credential 可区分，但公开错误不包含底层错误字符串。
- `GH953-AC5`：聚焦测试、格式检查和 Rust 编译检查通过。

## 边界情况

- `user_id = None` 的 key 不因缺少 owner 被拒绝。
- owner 查询失败属于基础设施错误，不得伪装成“key 无效”。
- 既有 Redis 快照只做兼容性清理，认证路径不读取其内容。

## 后续 schema 决策

现有外键在 canonical user 删除时会把 `api_keys.user_id` 置空，无法再区分“原本有 owner”与“创建时无 owner”。当前没有 canonical user 删除的应用入口；若未来增加该能力，必须先由维护者选择 `CASCADE`、`RESTRICT` 或显式撤销迁移，不能继续沿用 `SET NULL`。

## 开放问题

无。
