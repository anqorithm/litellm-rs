# Crush 库分析报告

> 分析目标：从 [charmbracelet/crush](https://github.com/charmbracelet/crush) 提取可借鉴的设计模式和最佳实践，用于优化 Sage 项目。

## 1. 项目概述

| 属性 | 值 |
|------|-----|
| **名称** | Crush |
| **语言** | Go |
| **Stars** | 19,466 |
| **定位** | 终端 AI 编码代理 |
| **核心特性** | 多模型支持、MCP/LSP 集成、会话管理 |

---

## 2. 值得借鉴的设计模式

### 2.1 Service 接口模式 ⭐⭐⭐⭐⭐

**Crush 实现**:
```go
type Service interface {
    pubsub.Subscriber[Session]
    Create(ctx context.Context, title string) (Session, error)
    Get(ctx context.Context, id string) (Session, error)
    List(ctx context.Context) ([]Session, error)
    Save(ctx context.Context, session Session) (Session, error)
    Delete(ctx context.Context, id string) error
}
```

**优点**:
- 清晰的契约定义
- 易于 Mock 测试
- 支持多种实现（内存、SQLite、远程）

**Sage 应用建议**:
- [ ] 为每个核心模块定义 trait 接口
- [ ] 实现 MockXxxService 用于测试

---

### 2.2 泛型 Pub/Sub 事件系统 ⭐⭐⭐⭐⭐

**Crush 实现**:
```go
type Broker[T any] struct {
    subs map[chan Event[T]]struct{}
    mu   sync.RWMutex
}

func (b *Broker[T]) Subscribe(ctx context.Context) <-chan Event[T]
func (b *Broker[T]) Publish(t EventType, payload T)
```

**优点**:
- 类型安全，无需类型断言
- 非阻塞发送，防止慢消费者阻塞
- 支持 context 取消

**Sage 应用建议**:
- [ ] 实现泛型 `EventBroker<T>`
- [ ] 用于 Agent 状态变更通知
- [ ] 用于工具执行结果广播

---

### 2.3 并发安全数据结构 (csync) ⭐⭐⭐⭐⭐

**Crush 实现**:
```go
// csync 包提供:
csync.Map[K, V]       // 并发安全 Map
csync.Slice[T]        // 并发安全 Slice
csync.Value[T]        // 并发安全值
csync.VersionedMap    // 带版本的 Map
```

**优点**:
- 统一的并发安全 API
- 避免到处使用 Mutex
- 版本化 Map 支持乐观锁

**Sage 应用建议**:
- [ ] 创建 `sage-core/src/sync/` 模块
- [ ] 实现 `ConcurrentMap<K, V>`
- [ ] 实现 `ConcurrentVec<T>`
- [ ] 实现 `AtomicValue<T>`

---

### 2.4 Coordinator 协调器模式 ⭐⭐⭐⭐

**Crush 实现**:
```go
type Coordinator interface {
    Run(ctx context.Context, sessionID, prompt string, attachments ...message.Attachment) (*fantasy.AgentResult, error)
    Cancel(sessionID string)
    CancelAll()
    IsSessionBusy(sessionID string) bool
    UpdateModels(ctx context.Context) error
}
```

**优点**:
- 统一管理多个 Agent 生命周期
- 支持取消和状态查询
- 解耦 Agent 创建和执行

**Sage 应用建议**:
- [ ] 实现 `AgentCoordinator` trait
- [ ] 管理 SubAgent 生命周期
- [ ] 支持并发 Agent 执行

---

### 2.5 Options 构造模式 ⭐⭐⭐⭐

**Crush 实现**:
```go
type SessionAgentOptions struct {
    LargeModel           Model
    SmallModel           Model
    SystemPromptPrefix   string
    IsSubAgent           bool
    DisableAutoSummarize bool
    IsYolo               bool
    Tools                []fantasy.AgentTool
}

func NewSessionAgent(opts SessionAgentOptions) SessionAgent
```

**优点**:
- 避免构造函数参数爆炸
- 支持可选参数和默认值
- 易于扩展

**Sage 应用建议**:
- [ ] 使用 Builder 模式或 `#[derive(Default)]`
- [ ] 为复杂配置提供 `XxxOptions` 结构

---

### 2.6 VCR 测试模式 ⭐⭐⭐⭐⭐

**Crush 实现**:
```go
func setupAgent(t *testing.T, pair modelPair) (SessionAgent, fakeEnv) {
    r := vcr.NewRecorder(t)  // 录制 HTTP 交互
    large, small := getModels(t, r, pair)
    // ...
}
```

**优点**:
- 录制真实 API 响应
- 回放时无需网络
- 确保测试可重复

**Sage 应用建议**:
- [ ] 集成 `wiremock-rs` 或自研 VCR
- [ ] 录制 LLM API 响应用于测试
- [ ] 支持 `--update` 更新录制

---

### 2.7 工具描述分离 ⭐⭐⭐⭐

**Crush 实现**:
```
tools/
├── edit.go      # 工具实现
├── edit.md      # 工具描述 (给 LLM 看)
├── bash.go
├── bash.tpl     # 模板化描述
```

**优点**:
- 代码与描述分离
- 描述可独立更新
- 支持模板化

**Sage 应用建议**:
- [ ] 工具描述使用 `.md` 文件
- [ ] 支持 `include_str!()` 嵌入
- [ ] 支持运行时加载

---

### 2.8 多模型测试矩阵 ⭐⭐⭐⭐

**Crush 实现**:
```go
var modelPairs = []modelPair{
    {"anthropic-sonnet", anthropicBuilder("claude-sonnet-4-5-20250929"), ...},
    {"openai-gpt-5", openaiBuilder("gpt-5"), ...},
    {"openrouter-kimi-k2", openRouterBuilder("moonshotai/kimi-k2-0905"), ...},
}
```

**优点**:
- 确保跨 Provider 兼容性
- 发现 Provider 特定问题
- CI 中可选择性运行

**Sage 应用建议**:
- [ ] 创建 `test_matrix!` 宏
- [ ] 支持 `--provider` 过滤
- [ ] 集成到 CI 矩阵

---

### 2.9 Panic 恢复机制 ⭐⭐⭐⭐

**Crush 实现**:
```go
func RecoverPanic(name string, cleanup func()) {
    if r := recover(); r != nil {
        event.Error(r, "panic", true, "name", name)
        filename := fmt.Sprintf("crush-panic-%s-%s.log", name, timestamp)
        // 写入 panic 日志
    }
}
```

**优点**:
- 生产环境稳定性
- 保留 panic 现场
- 支持清理回调

**Sage 应用建议**:
- [ ] 使用 `std::panic::catch_unwind`
- [ ] 记录 panic 到文件
- [ ] 发送遥测事件

---

### 2.10 LSP 深度集成 ⭐⭐⭐⭐⭐

**Crush 实现**:
```go
type LSPConfig struct {
    Command     string
    FileTypes   []string
    RootMarkers []string
    InitOptions map[string]any
}

// 工具执行后通知 LSP
func notifyLSPs(ctx context.Context, lspClients *csync.Map[string, *lsp.Client], filePath string)
```

**优点**:
- 获取代码诊断信息
- 支持引用查找
- 文件变更自动通知

**Sage 应用建议**:
- [ ] 集成 `tower-lsp` 客户端
- [ ] 提供 `diagnostics` 工具
- [ ] 提供 `references` 工具

---

## 3. 功能特性对比

| 特性 | Crush | Sage (当前) | 优先级 |
|------|-------|-------------|--------|
| 多 Provider 支持 | ✅ 10+ | ✅ 100+ | - |
| MCP 集成 | ✅ stdio/sse/http | ✅ | - |
| LSP 集成 | ✅ | ❌ | P1 |
| 泛型 Pub/Sub | ✅ | ❌ | P1 |
| 并发安全容器 | ✅ csync | ❌ | P1 |
| VCR 测试 | ✅ | ❌ | P2 |
| 工具描述分离 | ✅ | ❌ | P2 |
| Panic 恢复 | ✅ | ❌ | P2 |
| 多模型测试矩阵 | ✅ | ❌ | P3 |
| Skill 系统 | ✅ | ❌ | P3 |

---

## 4. 优化任务清单

### Phase 1: 核心基础设施 (P1)

- [ ] **Task 1.1**: 实现泛型 EventBroker
  - 文件: `sage-core/src/event/broker.rs`
  - 测试: 单元测试 + 并发测试

- [ ] **Task 1.2**: 实现并发安全容器
  - 文件: `sage-core/src/sync/mod.rs`
  - 包含: `ConcurrentMap`, `ConcurrentVec`, `AtomicValue`

- [ ] **Task 1.3**: 实现 AgentCoordinator
  - 文件: `sage-core/src/agent/coordinator.rs`
  - 功能: 生命周期管理、取消、状态查询

### Phase 2: 测试基础设施 (P2)

- [ ] **Task 2.1**: 集成 VCR 测试框架
  - 依赖: `wiremock` 或自研
  - 支持: 录制/回放模式

- [ ] **Task 2.2**: 工具描述分离
  - 格式: `.md` 文件
  - 加载: `include_str!()` 或运行时

- [ ] **Task 2.3**: Panic 恢复机制
  - 使用: `catch_unwind`
  - 日志: 写入文件 + 遥测

### Phase 3: 高级特性 (P3)

- [ ] **Task 3.1**: LSP 客户端集成
  - 依赖: `tower-lsp`
  - 工具: `diagnostics`, `references`

- [ ] **Task 3.2**: 多模型测试矩阵
  - 宏: `test_matrix!`
  - CI: GitHub Actions 矩阵

- [ ] **Task 3.3**: Skill 系统
  - 标准: Agent Skills 开放标准
  - 发现: 配置目录扫描

---

## 5. 架构对比图

```
┌─────────────────────────────────────────────────────────┐
│                     Crush 架构                          │
├─────────────────────────────────────────────────────────┤
│  cmd (CLI)                                              │
│    ↓                                                    │
│  app (DI Container)                                     │
│    ↓                                                    │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐                 │
│  │  agent  │  │   tui   │  │   lsp   │                 │
│  │Coordinat│  │BubbleTea│  │ Clients │                 │
│  └────┬────┘  └─────────┘  └─────────┘                 │
│       ↓                                                 │
│  ┌─────────────────────────────────────┐               │
│  │         Services Layer              │               │
│  │ session │ message │ permission      │               │
│  └─────────────────────────────────────┘               │
│       ↓                                                 │
│  ┌─────────────────────────────────────┐               │
│  │        Infrastructure               │               │
│  │  db │ pubsub │ csync │ config       │               │
│  └─────────────────────────────────────┘               │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│                     Sage 目标架构                        │
├─────────────────────────────────────────────────────────┤
│  sage-cli (CLI)                                         │
│    ↓                                                    │
│  sage-app (DI Container)                                │
│    ↓                                                    │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐                 │
│  │  agent  │  │   tui   │  │   lsp   │  ← 新增         │
│  │Coordinat│  │ Ratatui │  │ Clients │                 │
│  └────┬────┘  └─────────┘  └─────────┘                 │
│       ↓                                                 │
│  ┌─────────────────────────────────────┐               │
│  │         Services Layer              │               │
│  │ session │ message │ permission      │               │
│  └─────────────────────────────────────┘               │
│       ↓                                                 │
│  ┌─────────────────────────────────────┐               │
│  │        Infrastructure               │               │
│  │  db │ event │ sync │ config         │  ← 新增       │
│  └─────────────────────────────────────┘               │
└─────────────────────────────────────────────────────────┘
```

---

## 6. 参考资源

- [Crush GitHub](https://github.com/charmbracelet/crush)
- [Agent Skills 标准](https://agentskills.io)
- [Fantasy LLM 库](https://github.com/charmbracelet/fantasy)
- [Bubble Tea TUI](https://github.com/charmbracelet/bubbletea)
