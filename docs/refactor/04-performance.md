# 性能问题分析报告

## 概述

本报告对 litellm-rs 项目进行了系统性的性能分析，涵盖内存分配、锁竞争、异步阻塞、序列化效率、连接池配置和缓存策略六个维度。项目整体架构设计合理（使用了 DashMap、parking_lot、AtomicU64 等高性能原语），但在热路径上仍存在可优化的性能瓶颈。

---

## 1. 不必要的内存分配

### 1.1 大量 clone 调用

项目中共有约 **2523 个 `.clone()` 调用**，虽然部分是必要的（Arc clone、跨线程传递），但存在大量可避免的深拷贝。

#### 问题 1.1.1: CacheManager 热路径上的深拷贝 [高]

**文件**: `src/core/cache_manager/manager.rs:64,80,84`

```rust
// 第64行: L1 缓存命中时克隆整个 ChatCompletionResponse
return Ok(Some(entry.value.clone()));

// 第80行: L2 提升到 L1 时克隆 key 和 entry
l1.put(key.clone(), entry.clone());

// 第84行: L2 缓存命中时再次克隆 value
return Ok(Some(entry.value.clone()));
```

`ChatCompletionResponse` 是一个包含多层嵌套 Vec/String 的大型结构体，每次缓存命中都进行深拷贝。在高 QPS 场景下，这是严重的性能瓶颈。

**建议**: 将缓存值包装为 `Arc<ChatCompletionResponse>`，返回 `Arc::clone()` 而非深拷贝。L1/L2 之间的提升也只需 Arc clone。

---

#### 问题 1.1.2: FallbackConfig 每次查询都克隆整个 Vec [中]

**文件**: `src/core/router/fallback.rs:172-176`

```rust
pub fn get_fallbacks_for_type(&self, model_name: &str, fallback_type: FallbackType) -> Vec<String> {
    lock.read()
        .expect("FallbackConfig lock poisoned")
        .get(model_name)
        .cloned()           // 克隆整个 Vec<String>
        .unwrap_or_default()
}
```

每次路由决策都会克隆 fallback 列表。在高并发路由场景下产生不必要的堆分配。

**建议**: 将内部存储改为 `HashMap<String, Arc<Vec<String>>>`，返回 `Arc<Vec<String>>` 避免深拷贝。

---

#### 问题 1.1.3: Prometheus 指标 get_or_create 模式的冗余克隆 [中]

**文件**: `src/core/integrations/observability/prometheus.rs:272-281`

```rust
fn get_or_create_counter(map: &RwLock<HashMap<Labels, Arc<Counter>>>, labels: &Labels) -> Arc<Counter> {
    if let Some(counter) = map.read().get(labels).cloned() {  // 读路径: Arc clone (OK)
        return counter;
    }
    let mut write = map.write();
    write
        .entry(labels.clone())  // labels 被克隆用作 HashMap key
        .or_insert_with(|| Arc::new(Counter::default()))
        .clone()
}
```

每次创建新指标时 `labels.clone()` 会分配新的 HashMap。此函数在每个请求的指标记录路径上被调用。

**建议**: 使用 `entry` API 前先检查是否存在，或使用 `DashMap` 替代 `RwLock<HashMap>` 减少锁竞争的同时避免重复查找。

---

#### 问题 1.1.4: 观测指标记录中的重复 String 分配 [中]

**文件**: `src/core/observability/metrics.rs:121-154`

```rust
pub async fn record_request(&self, provider: &str, model: &str, ...) {
    let mut metrics = self.prometheus_metrics.write().await;
    let key = format!("{}:{}", provider, model);  // 每次请求都 format! 分配
    *metrics.request_total.entry(key.clone()).or_insert(0) += 1;
    // ...
    metrics.request_duration.entry(key.clone()).or_insert_with(...);  // 又一次 clone
    if !success {
        *metrics.error_total.entry(key.clone()).or_insert(0) += 1;   // 又一次 clone
    }
    // token usage 中还有更多 format!
    *metrics.token_usage.entry(format!("{}:prompt", key)).or_insert(0) += ...;
    *metrics.token_usage.entry(format!("{}:completion", key)).or_insert(0) += ...;
}
```

每个请求的指标记录路径上有 5+ 次 String 分配和 3+ 次 clone。

**建议**: 预计算 key 并使用 `SmallString` 或 intern 池；将 `key` 的多次 `entry()` 调用合并为一次查找；使用结构化 key 类型（如 `(ProviderId, ModelId)` 元组）替代字符串拼接。

---

#### 问题 1.1.5: estimate_size 通过序列化计算大小 [低]

**文件**: `src/core/cache_manager/manager.rs:152-157`

```rust
fn estimate_size(&self, response: &ChatCompletionResponse) -> usize {
    serde_json::to_string(response)
        .map(|s| s.len())
        .unwrap_or(1024)
}
```

为了估算大小而进行完整的 JSON 序列化，产生一个临时 String 后立即丢弃。

**建议**: 实现一个 `EstimateSize` trait，通过递归计算字段大小来估算，避免序列化开销。或使用 `serde_json::to_writer` 配合计数 writer。

---

#### 问题 1.1.6: provider_stats 读取时克隆整个 HashMap [低]

**文件**: `src/sdk/client/stats.rs:57`

```rust
self.provider_stats.read().await.clone()
```

返回整个 provider 统计信息的深拷贝。

**建议**: 返回 `Arc<HashMap>` 或按需查询单个 provider 的统计。

---

## 2. 锁竞争

### 2.1 锁使用概况

项目中共有 **355 处 RwLock/Mutex 使用**，分布在 72 个文件中。锁的类型分布：
- `parking_lot::RwLock` (同步): ~30 处，用于热路径（路由选择、指标、部署健康）
- `tokio::sync::RwLock` (异步): ~40 处，用于 I/O 密集型操作
- `std::sync::RwLock`: 3 处（`fallback.rs`, `teams/repository.rs`, `health/monitor.rs`）
- `std::sync::Mutex`: 6 处（`circuit_breaker.rs`）

#### 问题 2.1.1: CircuitBreaker 使用 std::sync::Mutex 并在 async 函数中持有 [高]

**文件**: `src/utils/error/recovery/circuit_breaker.rs:15-21,69-98`

```rust
pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,           // std::sync::Mutex
    last_failure_time: Arc<Mutex<Option<Instant>>>,  // std::sync::Mutex
    window_start: Arc<Mutex<Instant>>,         // std::sync::Mutex
}

async fn can_execute(&self) -> bool {
    let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
    match *state {
        CircuitState::Open => {
            if let Some(last_failure) = *self.last_failure_time.lock()... {
                // 同时持有两个 std::sync::Mutex 锁！
```

在 async 函数中使用 `std::sync::Mutex`，如果在 `.await` 点之间持有锁，会阻塞整个 tokio worker 线程。此外 `can_execute` 同时获取两个锁，存在死锁风险。

**建议**:
1. 将 `state` 和 `last_failure_time` 合并为单个结构体，用一把锁保护
2. 改用 `parking_lot::Mutex`（不会 poison，且性能更好）
3. 考虑用 `AtomicU8` 表示状态，`AtomicU64` 表示时间戳，完全无锁化

---

#### 问题 2.1.2: HealthMonitor 使用 std::sync::RwLock 在 async 上下文中 [高]

**文件**: `src/core/health/monitor.rs:47-50`

```rust
pub struct HealthMonitor {
    pub(crate) provider_health: Arc<RwLock<HashMap<String, ProviderHealth>>>,
    pub(crate) circuit_breakers: Arc<RwLock<HashMap<String, Arc<CircuitBreaker>>>>,
    pub(crate) check_tasks: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<()>>>>,
}
```

三个 `std::sync::RwLock` 在 async 方法中使用。`register_provider` 方法（第72-92行）依次获取 `provider_health` 和 `circuit_breakers` 的写锁，如果其中一个锁被长时间持有，会阻塞 tokio worker。

**建议**: 改用 `DashMap` 替代 `RwLock<HashMap>`，或至少改用 `tokio::sync::RwLock`。

---

#### 问题 2.1.3: FallbackConfig 使用 std::sync::RwLock 且有 4 把独立锁 [中]

**文件**: `src/core/router/fallback.rs:67-79`

```rust
pub struct FallbackConfig {
    general: RwLock<HashMap<String, Vec<String>>>,
    context_window: RwLock<HashMap<String, Vec<String>>>,
    content_policy: RwLock<HashMap<String, Vec<String>>>,
    rate_limit: RwLock<HashMap<String, Vec<String>>>,
}
```

4 把 `std::sync::RwLock` 保护 4 个独立的 HashMap。配置变更时需要分别获取 4 把写锁。

**建议**: 合并为单个结构体用一把锁保护，或改用 `parking_lot::RwLock`。配置是低频写、高频读的场景，也可考虑 `arc-swap` 实现无锁读。

---

#### 问题 2.1.4: Prometheus 指标 8 把 RwLock 的锁风暴 [高]

**文件**: `src/core/integrations/observability/prometheus.rs:250-258`

```rust
struct Metrics {
    requests_total: RwLock<HashMap<Labels, Arc<Counter>>>,
    requests_success: RwLock<HashMap<Labels, Arc<Counter>>>,
    requests_error: RwLock<HashMap<Labels, Arc<Counter>>>,
    input_tokens_total: RwLock<HashMap<Labels, Arc<Counter>>>,
    output_tokens_total: RwLock<HashMap<Labels, Arc<Counter>>>,
    cost_total: RwLock<HashMap<Labels, Arc<Counter>>>,
    request_latency: RwLock<HashMap<Labels, Arc<Histogram>>>,
    ttft_latency: RwLock<HashMap<Labels, Arc<Histogram>>>,
}
```

每个请求完成时需要更新多个指标，意味着要依次获取多把锁。`render_metrics` 导出时也要依次读取所有锁。

**建议**: 使用 `DashMap<Labels, MetricSet>` 将所有指标合并为单个并发 map，每个 label 组合对应一个 `MetricSet`（内部用 Atomic 计数器）。

---

#### 问题 2.1.5: record_request 持有 tokio::RwLock 写锁时间过长 [高]

**文件**: `src/core/observability/metrics.rs:121-154`

```rust
pub async fn record_request(&self, ...) {
    let mut metrics = self.prometheus_metrics.write().await;  // 获取写锁
    // ... 接下来 30+ 行代码都在写锁保护下执行
    // 包括多次 HashMap 查找、format!、条件判断等
}
```

写锁持有时间覆盖了整个方法体，所有并发请求的指标记录都被串行化。

**建议**: 使用 `DashMap` 或将指标改为 `AtomicU64` 计数器，完全避免写锁。

---

#### 问题 2.1.6: IpAccessControl 使用 tokio::RwLock 保护静态配置 [低]

**文件**: `src/core/ip_access/control.rs:15-19`

```rust
pub struct IpAccessControl {
    allowlist: Arc<RwLock<Vec<IpRule>>>,
    blocklist: Arc<RwLock<Vec<IpRule>>>,
    always_allow: Arc<RwLock<Vec<IpRule>>>,
}
```

IP 规则列表在运行时极少变更，但每次请求检查都需要获取读锁。

**建议**: 使用 `arc-swap::ArcSwap<Vec<IpRule>>` 实现无锁读取，仅在配置变更时原子替换。

---

#### 问题 2.1.7: RealtimeSession 6 把 parking_lot::RwLock [中]

**文件**: `src/core/realtime/session.rs:40-55`

```rust
pub struct RealtimeSession {
    state: RwLock<SessionState>,
    session_id: RwLock<Option<String>>,
    conversation_id: RwLock<Option<String>>,
    items: RwLock<HashMap<String, Item>>,
    tx: RwLock<Option<mpsc::Sender<ClientEvent>>>,
}
```

单个 session 有 5 把独立的 RwLock。`SessionState` 是 Copy 类型，完全可以用 AtomicU8。

**建议**: `state` 改用 `AtomicU8`；`session_id` 和 `conversation_id` 合并到一个结构体中用单把锁保护；`tx` 改用 `tokio::sync::watch`。

---

## 3. 异步代码中的阻塞操作

#### 问题 3.1: CircuitBreaker 在 async fn 中使用 std::sync::Mutex [高]

**文件**: `src/utils/error/recovery/circuit_breaker.rs:69-98`

已在 2.1.1 中详述。`can_execute` 和 `on_success`/`on_failure` 都是 async 函数，但内部使用 `std::sync::Mutex::lock()`。虽然当前实现中锁内没有 `.await`，但 `std::sync::Mutex` 的 guard 跨越 async 函数边界本身就是一个隐患——未来的修改可能意外引入 `.await`。

**建议**: 改用 `parking_lot::Mutex` 或完全无锁化。

---

#### 问题 3.2: std::thread::sleep 在非测试代码中出现 [中]

**文件**:
- `src/utils/logging/structured.rs:739,758,819` (测试代码)
- `src/core/integrations/observability/opentelemetry.rs:909` (测试代码)
- `src/core/models/mod.rs:338` (测试代码)

虽然当前所有 `std::thread::sleep` 调用都在 `#[cfg(test)]` 块中，但这仍然会阻塞 tokio 测试运行时的 worker 线程，导致测试执行变慢。

**建议**: 在 async 测试中使用 `tokio::time::sleep` 替代 `std::thread::sleep`。

---

#### 问题 3.3: SQLite fallback 中的同步文件系统操作 [中]

**文件**: `src/storage/database/seaorm_db/connection.rs:60-64`

```rust
async fn fallback_to_sqlite() -> Result<Self> {
    let data_dir = std::path::Path::new("data");
    if !data_dir.exists() {
        std::fs::create_dir_all(data_dir).map_err(|e| { ... })?;  // 同步 I/O
    }
}
```

在 async 函数中执行同步文件系统操作。

**建议**: 使用 `tokio::fs::create_dir_all` 替代。

---

#### 问题 3.4: Prometheus render_metrics 持有多把读锁进行字符串拼接 [中]

**文件**: `src/core/integrations/observability/prometheus.rs:319-457`

`render_metrics` 方法在持有 `RwLock` 读锁期间进行大量的 `format!` 字符串拼接操作。虽然不是阻塞 I/O，但长时间持有读锁会阻止写入（指标更新）。

**建议**: 先在锁内快速收集数据到临时结构，释放锁后再进行字符串格式化。

---

#### 问题 3.5: 系统指标收集使用 parking_lot::Mutex 保护全局状态 [低]

**文件**: `src/monitoring/metrics/system.rs:14-23`

```rust
static SYSTEM: Lazy<parking_lot::Mutex<System>> = ...;
static NETWORKS: Lazy<parking_lot::Mutex<Networks>> = ...;
static DISKS: Lazy<parking_lot::Mutex<Disks>> = ...;
```

`sysinfo` 的 `System::refresh_*` 方法可能耗时较长，在 async 上下文中持有 Mutex 会阻塞 worker。

**建议**: 将系统指标收集移到 `tokio::task::spawn_blocking` 中执行。

---

## 4. 序列化效率

项目中共有约 **2025 处 serde_json 调用**（包括 `to_string`、`from_str`、`to_value`、`from_value` 等）。

#### 问题 4.1: 缓存大小估算使用完整 JSON 序列化 [高]

**文件**: `src/core/cache_manager/manager.rs:152-157`

已在 1.1.5 中详述。每次缓存写入都进行一次完整的 JSON 序列化仅为了计算字节数。

**建议**: 实现 `fn estimate_size(&self) -> usize` trait，基于字段类型递归估算。

---

#### 问题 4.2: Prometheus 指标导出使用大量 format! 拼接 [中]

**文件**: `src/core/integrations/observability/prometheus.rs:319-457`

```rust
pub fn render_metrics(&self) -> String {
    let mut output = String::new();
    // 30+ 次 format! 调用，每次都分配临时 String
    output.push_str(&format!("# HELP {}_{} {}", prefix, name, help));
    output.push_str(&format!("# TYPE {}_{} counter", prefix, name));
    // ...
}
```

每次 `/metrics` 端点调用都进行大量字符串分配。

**建议**: 使用 `write!(&mut output, ...)` 直接写入 String，避免中间 format! 分配。预估 output 容量并 `String::with_capacity()`。

---

#### 问题 4.3: 未使用 simd-json 或 sonic-rs 等高性能 JSON 库 [低]

项目完全依赖 `serde_json` 进行 JSON 序列化/反序列化。对于 AI Gateway 这种 JSON 密集型应用，JSON 处理占据了显著的 CPU 时间。

**建议**: 在热路径（请求/响应处理）上评估 `simd-json` 或 `sonic-rs` 的性能收益。这些库在解析大型 JSON（如 LLM 响应）时可提供 2-4x 的性能提升。

---

#### 问题 4.4: audit 事件序列化在热路径上 [低]

**文件**: `src/core/audit/events.rs:220-225`

```rust
pub fn to_json(&self) -> serde_json::Result<String> {
    serde_json::to_string(self)
}
pub fn to_json_pretty(&self) -> serde_json::Result<String> {
    serde_json::to_string_pretty(self)
}
```

审计事件在每个请求上序列化。如果审计功能开启，这会成为热路径上的开销。

**建议**: 使用异步通道将审计事件发送到后台线程进行序列化和写入，避免阻塞请求处理。

---

## 5. 连接池配置

#### 问题 5.1: 存在两个独立的全局 HTTP 客户端单例 [高]

**文件**:
- `src/utils/net/http.rs:59` — `SHARED_HTTP_CLIENT` (pool_max_idle_per_host=100)
- `src/core/providers/base/connection_pool.rs:48` — `GLOBAL_CLIENT` (pool_max_idle_per_host=80)

两个独立的全局 HTTP 客户端，各自维护独立的连接池。这意味着：
1. 同一个上游 provider 可能有两套 TCP 连接
2. 连接池资源浪费（总共 180 个 idle 连接/host）
3. DNS 缓存不共享

```rust
// http.rs
static SHARED_HTTP_CLIENT: OnceLock<Client> = OnceLock::new();
// pool_max_idle_per_host: 100, timeout: 30s

// connection_pool.rs
static GLOBAL_CLIENT: LazyLock<Arc<Client>> = LazyLock::new(|| {
    // pool_max_idle_per_host: 80, timeout: 600s
});
```

超时配置也不一致（30s vs 600s）。

**建议**: 统一为单个全局 HTTP 客户端，通过 `get_client_with_timeout()` 提供不同超时的变体。

---

#### 问题 5.2: BaseHttpClient 每个 provider 实例创建独立的 reqwest::Client [中]

**文件**: `src/core/providers/base_provider.rs:127-141`

```rust
impl BaseHttpClient {
    pub fn new(config: BaseProviderConfig) -> Result<Self, ProviderError> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(timeout))
            .pool_max_idle_per_host(10)  // 每个 provider 只有 10 个 idle 连接
            .build()?;
        Ok(Self { client, config })
    }
}
```

每个 provider 实例创建独立的 `reqwest::Client`，各自有独立的连接池（仅 10 个 idle 连接/host）。如果有 50 个 provider 实例，就有 50 个独立的连接池。

**建议**: 使用全局 `GLOBAL_CLIENT`，仅在需要特殊配置（如自定义 TLS）时创建独立客户端。

---

#### 问题 5.3: 数据库连接池默认值偏小 [低]

**文件**: `src/config/models/mod.rs:112-120`

```rust
pub fn default_max_connections() -> u32 { 10 }
pub fn default_redis_max_connections() -> u32 { 20 }
```

对于高吞吐量网关（目标 10,000+ RPS），PostgreSQL 默认 10 个连接和 Redis 默认 20 个连接可能不足。

**建议**: 根据 CPU 核心数动态计算默认值，如 `num_cpus * 2` 用于 PostgreSQL，`num_cpus * 4` 用于 Redis。

---

#### 问题 5.4: Redis 连接池使用 tokio::RwLock 保护 Vec [中]

**文件**: `src/storage/redis_optimized/connection.rs:14-22`

```rust
pub(super) struct ConnectionPool {
    pub(super) connections: Arc<RwLock<Vec<PooledConnection>>>,
    pub(super) semaphore: Arc<Semaphore>,
}
```

获取连接时需要获取写锁（`connections.write().await`），所有连接获取操作被串行化。Semaphore 限制了并发数，但锁竞争仍然是瓶颈。

**建议**: 使用 `crossbeam::queue::ArrayQueue` 或 `tokio::sync::mpsc` 实现无锁连接池。

---

## 6. 缓存策略

#### 问题 6.1: L1 缓存使用 RwLock<LruCache> 导致读写都需要写锁 [高]

**文件**: `src/core/cache_manager/manager.rs:22,57-69`

```rust
pub struct CacheManager {
    l1_cache: Arc<RwLock<LruCache<CacheKey, CacheEntry<ChatCompletionResponse>>>>,
}

pub async fn get(&self, key: &CacheKey) -> Result<Option<ChatCompletionResponse>> {
    {
        let mut l1 = self.l1_cache.write();  // 读操作也需要写锁！
        if let Some(entry) = l1.get_mut(key) {  // LRU 的 get 会修改内部顺序
```

LRU 缓存的 `get` 操作会更新访问顺序，因此即使是读操作也需要获取写锁。这意味着所有缓存查询都被串行化，在高 QPS 下成为严重瓶颈。

**建议**: 使用 `moka::sync::Cache` 或 `quick_cache::Cache` 替代，这些库内部使用分段锁和无锁并发策略，支持真正的并发读取。

---

#### 问题 6.2: 语义缓存未实现 [中]

**文件**: `src/core/cache_manager/manager.rs:132-148`

```rust
async fn semantic_lookup(&self, _key: &CacheKey) -> Result<Option<ChatCompletionResponse>> {
    // TODO: Implement semantic similarity search
    Ok(None)
}

async fn update_semantic_cache(&self, _key: &CacheKey) -> Result<()> {
    // TODO: Implement semantic cache updates
    Ok(())
}
```

语义缓存是 AI Gateway 的关键性能优化点（相似请求可以复用响应），但当前完全未实现。`enable_semantic` 配置项存在但无实际效果。

**建议**: 集成向量数据库（项目已有 Qdrant 支持）实现语义缓存，或使用本地 embedding + 近似最近邻搜索。

---

#### 问题 6.3: 缓存清理策略基于计数触发 [低]

**文件**: `src/core/cache_manager/manager.rs:123-126`

```rust
// Cleanup expired entries periodically
if self.l2_cache.len().is_multiple_of(1000) {
    self.cleanup_expired().await;
}
```

每 1000 次写入触发一次清理。问题：
1. 如果写入速率低，过期条目会长时间占用内存
2. 如果写入速率高，清理频率可能过高
3. `DashMap::len()` 本身有一定开销（需要遍历所有分片）

**建议**: 使用后台定时任务进行清理（如每 60 秒），或使用 `moka` 等自带过期清理的缓存库。

---

#### 问题 6.4: VirtualKeyManager 缓存无 TTL 和大小限制 [中]

**文件**: `src/core/virtual_keys/manager.rs:16-21`

```rust
struct KeyManagerData {
    cache: HashMap<String, VirtualKey>,
    rate_limits: HashMap<String, RateLimitState>,
}
```

key 缓存是一个无限增长的 HashMap，没有 TTL、没有大小限制、没有 LRU 淘汰。长时间运行后可能导致内存泄漏。

**建议**: 使用 `moka::sync::Cache` 或 `lru::LruCache` 替代，设置合理的 TTL 和最大容量。

---

#### 问题 6.5: PricingService 缓存无过期机制 [低]

**文件**: `src/services/pricing/service.rs:36`

```rust
pricing_data: Arc::new(RwLock::new(PricingData { ... }))
```

定价数据加载后永不过期。如果上游定价发生变化，需要重启服务才能更新。

**建议**: 添加定时刷新机制（如每小时从远程 URL 重新加载）。

---

## 7. 综合优先级排序

| 优先级 | 问题 | 影响范围 | 预估收益 |
|--------|------|----------|----------|
| P0 | 2.1.5 record_request 写锁持有过长 | 每个请求 | 消除指标记录的串行化瓶颈 |
| P0 | 6.1 L1 缓存读操作需要写锁 | 每个缓存查询 | 缓存查询并发度提升 10x+ |
| P0 | 5.1 两个全局 HTTP 客户端 | 所有外部请求 | 减少连接数，提升连接复用率 |
| P1 | 2.1.1 CircuitBreaker std::sync::Mutex | 每个 provider 调用 | 消除 async 中的阻塞风险 |
| P1 | 2.1.4 Prometheus 8 把 RwLock | 每个请求 | 指标记录性能提升 5x+ |
| P1 | 1.1.1 CacheManager 深拷贝 | 每个缓存命中 | 减少大对象拷贝开销 |
| P1 | 5.2 每个 provider 独立连接池 | 所有 provider 请求 | 连接复用率提升 |
| P2 | 1.1.4 指标记录重复 String 分配 | 每个请求 | 减少 5+ 次/请求的堆分配 |
| P2 | 2.1.2 HealthMonitor std::sync::RwLock | 健康检查路径 | 消除 async 阻塞风险 |
| P2 | 4.2 Prometheus format! 拼接 | 每次指标导出 | 减少临时 String 分配 |
| P2 | 5.4 Redis 连接池锁竞争 | 每个 Redis 操作 | 连接获取延迟降低 |
| P2 | 6.4 VirtualKey 缓存无限增长 | 长期运行 | 防止内存泄漏 |
| P3 | 其余低严重度问题 | 各自场景 | 渐进式优化 |
