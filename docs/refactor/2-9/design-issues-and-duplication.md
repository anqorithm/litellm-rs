# 设计问题与重复设计清单（2-9）

> 基于当前代码库扫描结果整理，聚焦“架构问题 + 重复实现”。

## 1. 架构层面的关键问题

### 1.1 Provider 体系“声明能力”与“可调度能力”不一致
- `src/core/providers/mod.rs` 中 provider 模块声明很多，但统一 `Provider` enum / factory 覆盖有限。
- 结果：新增 provider 成本高（多点修改），且外部很难判断真实可用面。

### 1.2 Router 双轨并存（legacy + unified）
- `AppState` 同时持有 `ProviderRegistry` 与 `UnifiedRouter`，AI 路由仍大量直连 registry。
- 结果：迁移状态长期悬空，行为路径不唯一。

### 1.3 分层边界破坏（core 依赖 HTTP/Actix）
- `core/budget|guardrails|ip_access` 中含 Actix middleware。
- 结果：领域层与传输层耦合，测试和演进成本升高。

### 1.4 路由定义与实际挂载不一致
- `server/http.rs` 只挂载部分路由，但 `server/routes/*` 存在大量未接入模块。
- 结果：文档/代码与运行时行为偏差。

### 1.5 运行时占位实现仍在关键路径
- models/health 等模块存在 TODO、固定值返回。
- 结果：接口可见但语义不可靠，运维与集成侧容易误判。

### 1.6 请求级临时仓库构建（状态生命周期异常）
- keys/teams handler 中按请求新建 in-memory manager/repo。
- 结果：状态不可持续，与 app-level state 设计冲突。

### 1.7 Fat Handler / God Module
- `server/routes/ai/chat.rs` 体积过大，混合协议转换、业务编排、流式处理。
- 结果：单点复杂度高，回归风险大。

---

## 2. 重复设计与重复代码（高频）

### 2.1 Provider 构造模板重复
大量 provider 的 `new(config)` 都是：
`validate -> GlobalPoolManager::new -> load models -> Ok(Self)`。

### 2.2 Header 构建逻辑重复
`get_request_headers()` 在多个 provider 内近似复制（Bearer + custom headers）。

### 2.3 Chat 非流式/流式执行流水线重复
- URL/body 构建
- execute_request
- status + bytes + map error
- transform response / stream
在多 provider 重复出现。

### 2.4 Health Check 与 Cost 计算模板重复
- “有 API key 即 healthy”
- usage 组装后调用 pricing
重复分布在多个 provider。

### 2.5 Router 策略重复实现
`strategy/selection.rs` 与 `strategy_impl.rs` 同时维护 round-robin / least-latency / least-cost / weighted 等。

### 2.6 Fallback 新旧两套并行
`core/router/fallback.rs` 与 `core/router/load_balancer/fallback_*` 语义重复。

### 2.7 测试重复（fallback / config tests）
- fallback 语义在两套测试文件重复验证。
- config 模型测试存在大量模板化重复（default/serde/merge/clone）。

---

## 3. 优先级建议（短版）

### P0（先收敛）
1. 收敛路由与 provider 主路径（只保留一条 runtime 主干）。
2. 把 HTTP middleware 从 core 抽离回 server 层。
3. 清理“声明但未挂载/未完成”的路由与处理器。

### P1（降重复）
1. 统一 provider 基类能力：构造、headers、http error map、stream/non-stream pipeline。
2. 合并 router/fallback 双实现。
3. 去重测试模板，保留行为覆盖而非样板堆积。

### P2（可维护性）
1. 拆分 fat handler。
2. 明确模块边界与依赖方向（transport -> app -> domain -> infra）。
