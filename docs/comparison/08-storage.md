# Storage System Comparison: litellm-rs vs litellm

This document provides an in-depth comparison of the storage systems between the Rust implementation (litellm-rs) and the Python implementation (litellm).

## Executive Summary

| Feature | litellm-rs (Rust) | litellm (Python) |
|---------|-------------------|------------------|
| **Primary Database** | PostgreSQL + SQLite | PostgreSQL (via Prisma) |
| **ORM** | SeaORM | Prisma |
| **Cache System** | Redis + In-Memory | Redis + In-Memory + Disk + Cloud |
| **Semantic Cache** | Vector DB (separate) | Redis/Qdrant Semantic Cache |
| **Object Storage** | S3 + Local | S3 + GCS + Azure Blob |
| **Vector DB** | Qdrant, Pinecone, Weaviate | Qdrant (semantic cache) |
| **Architecture** | Unified StorageLayer | Modular Cache Classes |

---

## 1. Database Support Comparison

### 1.1 PostgreSQL

#### litellm-rs (Rust)

```rust
// src/storage/database/seaorm_db/connection.rs
pub struct DatabaseConfig {
    pub url: String,                    // postgresql://localhost/litellm
    pub max_connections: u32,           // default: 10
    pub connection_timeout: u64,        // default: 5 seconds
    pub ssl: bool,                      // SSL support
    pub enabled: bool,                  // enable/disable
}
```

**Features:**
- SeaORM as the ORM layer
- Connection pooling with configurable limits
- Automatic migrations via `sea_orm_migration`
- SQLite automatic fallback when PostgreSQL unavailable
- Connection health checks

**Implementation:**
```rust
impl SeaOrmDatabase {
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        match Self::try_connect(&config.url, config).await {
            Ok(db) => Ok(Self { db, backend_type }),
            Err(e) => {
                // Fallback to SQLite if PostgreSQL fails
                if config.url.starts_with("postgresql://") {
                    Self::fallback_to_sqlite().await
                } else {
                    Err(e)
                }
            }
        }
    }
}
```

#### litellm (Python)

```python
# litellm/proxy/db/prisma_client.py
class PrismaWrapper:
    def __init__(self, original_prisma, iam_token_db_auth: bool):
        self._original_prisma = original_prisma
        self.iam_token_db_auth = iam_token_db_auth
```

**Features:**
- Prisma ORM with schema-first approach
- AWS RDS IAM token authentication support
- Automatic token refresh (15-minute tokens with 3-minute buffer)
- Database migrations via `prisma migrate` or `prisma db push`

**RDS IAM Token Handling:**
```python
async def _token_refresh_loop(self):
    """Background loop for proactive token refresh"""
    while True:
        sleep_seconds = self._calculate_seconds_until_refresh()
        if sleep_seconds > 0:
            await asyncio.sleep(sleep_seconds)
        await self._safe_refresh_token()
```

### 1.2 SQLite

#### litellm-rs (Rust)

**Native SQLite support with automatic fallback:**

```rust
// Automatic fallback when PostgreSQL unavailable
async fn fallback_to_sqlite() -> Result<Self> {
    let db_path = default_sqlite_path(); // XDG-compatible, env var override
    let sqlite_path = format!("sqlite://{}?mode=rwc", db_path.display());
    let mut opt = ConnectOptions::new(sqlite_path.to_string());
    opt.max_connections(5)
       .sqlx_logging(true);
    let db = Database::connect(opt).await?;
    Ok(Self { db, backend_type: DatabaseBackendType::SQLite })
}
```

**Key Point:** SQLite is built-in as a fallback, not a primary database.

#### litellm (Python)

- **No native SQLite support** in the main codebase
- PostgreSQL is the only officially supported database
- DynamoDB support is deprecated

### 1.3 MySQL

| Aspect | litellm-rs | litellm |
|--------|------------|---------|
| Support | Not implemented | Not supported |
| Notes | SeaORM supports MySQL, could be added | PostgreSQL-only architecture |

### 1.4 Other Databases

#### litellm-rs
- SQLite (fallback)
- PostgreSQL (primary)
- Architecture supports adding more via SeaORM

#### litellm (Python)
- PostgreSQL (primary via Prisma)
- DynamoDB (deprecated)
```python
# litellm/proxy/db/dynamo_db.py
"""Deprecated. Only PostgreSQL is supported."""
class DynamoDBWrapper(CustomDB):
    # Legacy DynamoDB support with STS assume role
```

---

## 2. Cache System Comparison

### 2.1 Redis Support

#### litellm-rs (Rust)

**Architecture:**
```
src/storage/redis/
  mod.rs         # Module exports
  pool.rs        # Connection pooling (MultiplexedConnection)
  cache.rs       # Basic cache operations
  batch.rs       # Batch operations (mget, mset)
  collections.rs # List and Set operations
  hash.rs        # Hash and Sorted Set operations
  pubsub.rs      # Pub/Sub operations
  atomic.rs      # Atomic operations
```

**RedisPool Implementation:**
```rust
pub struct RedisPool {
    pub(crate) client: Option<Client>,
    pub(crate) connection_manager: Option<MultiplexedConnection>,
    pub(crate) config: RedisConfig,
    pub(crate) noop_mode: bool,  // Graceful degradation
}

impl RedisPool {
    pub async fn new(config: &RedisConfig) -> Result<Self> {
        let client = Client::open(config.url.as_str())?;
        let connection_manager = client.get_multiplexed_async_connection().await?;
        Ok(Self { client: Some(client), connection_manager: Some(connection_manager), ... })
    }

    // Graceful fallback when Redis unavailable
    pub fn create_noop() -> Self {
        Self { client: None, connection_manager: None, noop_mode: true, ... }
    }
}
```

**Supported Operations:**
- Basic: GET, SET, DELETE, EXISTS, EXPIRE, TTL
- Batch: MGET, MSET
- Collections: LPUSH, LPOP, SADD, SREM, SMEMBERS
- Hash: HSET, HGET, HDEL, HGETALL
- Pub/Sub: PUBLISH, SUBSCRIBE

#### litellm (Python)

**Architecture:**
```python
# litellm/caching/redis_cache.py
class RedisCache(BaseCache):
    def __init__(self, host, port, password, redis_flush_size, namespace, ...):
        self.redis_client = get_redis_client(**redis_kwargs)
        self.redis_async_client = None  # Lazy initialization
        self.async_redis_conn_pool = get_redis_connection_pool(**redis_kwargs)
        self.redis_batch_writing_buffer = []  # High-traffic buffering
```

**Advanced Features:**
- Redis Cluster support (`RedisClusterCache`)
- Batch cache writing with flush buffer
- Pipeline operations for bulk writes
- Service health monitoring integration
- GCP IAM authentication for managed Redis

**Cluster Support:**
```python
# litellm/caching/redis_cluster_cache.py
class RedisClusterCache(RedisCache):
    # Extends base Redis with cluster-specific operations
```

### 2.2 In-Memory Cache

#### litellm-rs (Rust)

**No dedicated in-memory cache module.** Redis with noop mode serves as fallback:
```rust
// When Redis unavailable, operations become no-ops
if self.noop_mode {
    return Ok(());  // Graceful degradation
}
```

#### litellm (Python)

**Full-featured In-Memory Cache:**
```python
# litellm/caching/in_memory_cache.py
class InMemoryCache(BaseCache):
    def __init__(self, max_size_in_memory=200, default_ttl=600, max_size_per_item=1024):
        self.cache_dict: dict = {}
        self.ttl_dict: dict = {}
        self.expiration_heap: list[tuple[float, str]] = []  # Min-heap for eviction
```

**Features:**
- Configurable size limits (200 items default)
- TTL-based expiration with heap-based eviction
- Per-item size checking (1MB default limit)
- LRU-like eviction when full

**Eviction Policy:**
```python
def evict_cache(self):
    """
    1. Remove expired items
    2. If still full, evict items with earliest expiration
    """
    current_time = time.time()
    while self.expiration_heap:
        expiration_time, key = self.expiration_heap[0]
        if expiration_time <= current_time:
            heapq.heappop(self.expiration_heap)
            self._remove_key(key)
        else:
            break
```

### 2.3 Semantic Cache

#### litellm-rs (Rust)

**No built-in semantic cache.** Vector storage is separate:
```rust
// src/storage/vector/mod.rs
pub enum VectorStoreBackend {
    Qdrant(QdrantStore),
    Weaviate(WeaviateStore),
    Pinecone(PineconeStore),
}
```

Semantic caching would need to be implemented at the application layer using the vector store.

#### litellm (Python)

**Multiple Semantic Cache Implementations:**

**1. Redis Semantic Cache:**
```python
# litellm/caching/redis_semantic_cache.py
class RedisSemanticCache(BaseCache):
    def __init__(self, similarity_threshold, embedding_model="text-embedding-ada-002"):
        from redisvl.extensions.llmcache import SemanticCache
        self.distance_threshold = 1 - similarity_threshold
        self.llmcache = SemanticCache(
            name=index_name,
            redis_url=redis_url,
            vectorizer=CustomTextVectorizer(self._get_embedding),
            distance_threshold=self.distance_threshold
        )
```

**2. Qdrant Semantic Cache:**
```python
# litellm/caching/qdrant_semantic_cache.py
class QdrantSemanticCache(BaseCache):
    def __init__(self, qdrant_api_base, qdrant_api_key, collection_name,
                 similarity_threshold, quantization_config, embedding_model):
        # Supports binary, scalar, or product quantization
        # Automatic collection creation
```

**Features:**
- Configurable similarity threshold
- Multiple embedding models
- Async support with router integration
- Quantization for storage efficiency

---

## 3. Object Storage Comparison

### 3.1 S3 Support

#### litellm-rs (Rust)

**Location:** `src/storage/files/s3.rs`

```rust
pub struct S3Storage {
    bucket: String,
    region: String,
    #[cfg(feature = "s3")]
    client: Option<aws_s3::Client>,
}

impl S3Storage {
    pub async fn new(config: &S3Config) -> Result<Self> {
        #[cfg(feature = "s3")]
        {
            let region = Region::new(config.region.clone());
            let aws_config = aws_config::defaults(BehaviorVersion::latest())
                .region(region).load().await;
            let client = aws_s3::Client::new(&aws_config);
            Ok(Self { bucket, region, client: Some(client) })
        }
    }
}
```

**Operations:**
- `store(filename, content)` - Upload with UUID
- `get(file_id)` - Download
- `delete(file_id)` - Remove
- Feature-gated (`#[cfg(feature = "s3")]`)

#### litellm (Python)

**Location:** `litellm/caching/s3_cache.py`

```python
class S3Cache(BaseCache):
    def __init__(self, s3_bucket_name, s3_region_name, s3_endpoint_url, ...):
        self.s3_client = boto3.client("s3",
            region_name=s3_region_name,
            endpoint_url=s3_endpoint_url,  # S3-compatible support
            ...
        )
```

**Features:**
- Cache-Control headers
- Expiration time support
- S3-compatible endpoints (MinIO, etc.)
- Async via `run_in_executor`

### 3.2 Local Storage

#### litellm-rs (Rust)

**Full Implementation:** `src/storage/files/local.rs`

```rust
pub struct LocalStorage {
    base_path: PathBuf,
}

impl LocalStorage {
    pub async fn store(&self, filename: &str, content: &[u8]) -> Result<String> {
        let file_id = Uuid::new_v4().to_string();
        let file_path = self.get_file_path(&file_id);
        // Subdirectory distribution for better performance
        let subdir = &file_id[..2];  // First 2 chars as subdir
        // Store both file and metadata
        self.store_metadata(&file_id, &metadata).await?;
        Ok(file_id)
    }
}
```

**Features:**
- UUID-based file IDs
- Subdirectory distribution (first 2 chars)
- Metadata storage (`.meta` files)
- Content type detection
- SHA256 checksum calculation
- Health checks

#### litellm (Python)

**Disk Cache:** `litellm/caching/disk_cache.py`
```python
class DiskCache(BaseCache):
    def __init__(self, disk_cache_dir=None):
        # Simpler implementation for caching purposes
```

### 3.3 Cloud Storage

#### litellm-rs
- S3 only (feature-gated)
- No GCS or Azure Blob

#### litellm (Python)
- **S3Cache** - AWS S3
- **GCSCache** - Google Cloud Storage
- **AzureBlobCache** - Azure Blob Storage

```python
# litellm/caching/gcs_cache.py
class GCSCache(BaseCache):
    def __init__(self, bucket_name, path_service_account, gcs_path):
        # Google Cloud Storage integration

# litellm/caching/azure_blob_cache.py
class AzureBlobCache(BaseCache):
    def __init__(self, account_url, container):
        # Azure Blob Storage integration
```

---

## 4. Vector Database Comparison

### 4.1 Qdrant

#### litellm-rs (Rust)

**Full Implementation:** `src/storage/vector/qdrant.rs`

```rust
pub struct QdrantStore {
    url: String,
    api_key: Option<String>,
    collection: String,
    client: reqwest::Client,  // HTTP-based
}

impl QdrantStore {
    pub async fn store(&self, id: &str, vector: &[f32], metadata: Option<Value>) -> Result<()>;
    pub async fn search(&self, query_vector: &[f32], limit: usize, threshold: Option<f32>) -> Result<Vec<SearchResult>>;
    pub async fn batch_store(&self, points: &[VectorPoint]) -> Result<()>;
    pub async fn count(&self) -> Result<u64>;
}
```

**Features:**
- HTTP REST API client
- Collection auto-creation
- Cosine distance by default
- Batch operations
- Health checks

#### litellm (Python)

**As Semantic Cache:** `litellm/caching/qdrant_semantic_cache.py`

```python
class QdrantSemanticCache(BaseCache):
    def __init__(self, qdrant_api_base, qdrant_api_key, collection_name,
                 similarity_threshold, quantization_config, embedding_model):
        # Three quantization modes: binary, scalar, product
```

**Key Differences:**
| Aspect | litellm-rs | litellm |
|--------|------------|---------|
| Purpose | General vector storage | Semantic caching |
| Quantization | Not configurable | Binary/Scalar/Product |
| Embedding | External | Built-in with litellm |
| Integration | Standalone | Router-aware |

### 4.2 Other Vector Databases

#### litellm-rs (Rust)

**Supported via Backend Enum:**
```rust
pub enum VectorStoreBackend {
    Qdrant(QdrantStore),
    Weaviate(WeaviateStore),
    Pinecone(PineconeStore),
}
```

**Weaviate:** `src/storage/vector/weaviate.rs`
**Pinecone:** `src/storage/vector/pinecone.rs`

#### litellm (Python)

- Qdrant only (for semantic cache)
- No Weaviate/Pinecone integration for caching

---

## 5. Data Model Comparison

### 5.1 ORM Usage

#### litellm-rs (Rust) - SeaORM

**Entities:** `src/storage/database/entities/`
```rust
// user.rs
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime,
}

// batch.rs, password_reset_token.rs, user_session.rs, pricing.rs
```

**Advantages:**
- Compile-time type safety
- Async-first design
- Both PostgreSQL and SQLite support
- Active Record + Query Builder patterns

#### litellm (Python) - Prisma

**Schema-First Approach:**
```prisma
// schema.prisma
model LiteLLM_Key {
  id        String   @id @default(uuid())
  key       String   @unique
  team_id   String?
  budget    Float?
  created_at DateTime @default(now())
}
```

**Advantages:**
- Auto-generated types
- Migration management
- Type-safe queries via generated client

### 5.2 Migration Mechanism

#### litellm-rs (Rust)

**SeaORM Migrations:** `src/storage/database/migration/`

```rust
pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_users_table::Migration),
            Box::new(m20240101_000002_create_password_reset_tokens_table::Migration),
            Box::new(m20240101_000003_create_batches_table::Migration),
            Box::new(m20240101_000004_create_user_sessions_table::Migration),
        ]
    }
}

// Example migration
impl MigrationName for Migration {
    fn name(&self) -> &str { "m20240101_000001_create_users_table" }
}
```

**Execution:**
```rust
pub async fn migrate(&self) -> Result<()> {
    Migrator::up(&self.db, None).await?;
    Ok(())
}
```

#### litellm (Python)

**Prisma Migrations:**
```python
class PrismaManager:
    @staticmethod
    def setup_database(use_migrate: bool = False) -> bool:
        if use_migrate:
            # Use prisma migrate for production
            ProxyExtrasDBManager.setup_database(use_migrate=True)
        else:
            # Use prisma db push for development
            subprocess.run(["prisma", "db", "push", "--accept-data-loss"])
```

### 5.3 Data Structures

#### litellm-rs (Rust)

```rust
// Cache data types
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub metadata: Option<serde_json::Value>,
    pub vector: Option<Vec<f32>>,
}

pub struct VectorPoint {
    pub id: String,
    pub vector: Vec<f32>,
    pub metadata: Option<serde_json::Value>,
}

pub struct FileMetadata {
    pub id: String,
    pub filename: String,
    pub content_type: String,
    pub size: u64,
    pub created_at: DateTime<Utc>,
    pub checksum: String,
}
```

#### litellm (Python)

```python
# Cache types
class CachedEmbedding(TypedDict):
    embedding: List[float]
    index: int
    object: str
    model: Optional[str]

# Base cache interface
class BaseCache(ABC):
    def set_cache(self, key, value, **kwargs): pass
    async def async_set_cache(self, key, value, **kwargs): pass
    def get_cache(self, key, **kwargs): pass
    async def async_get_cache(self, key, **kwargs): pass
```

---

## 6. Architectural Differences

### 6.1 Overall Architecture

#### litellm-rs - Unified StorageLayer

```rust
pub struct StorageLayer {
    pub database: Arc<database::Database>,
    pub redis: Arc<redis::RedisPool>,
    pub files: Arc<files::FileStorage>,
    pub vector: Option<Arc<vector::VectorStoreBackend>>,
}
```

**Characteristics:**
- Single entry point for all storage
- Arc-wrapped for thread-safe sharing
- Health checks unified
- Graceful degradation built-in

#### litellm (Python) - Modular Cache Classes

```python
class Cache:
    def __init__(self, type: LiteLLMCacheType = LiteLLMCacheType.LOCAL, ...):
        if type == LiteLLMCacheType.REDIS:
            self.cache = RedisCache(...)
        elif type == LiteLLMCacheType.QDRANT_SEMANTIC:
            self.cache = QdrantSemanticCache(...)
        elif type == LiteLLMCacheType.S3:
            self.cache = S3Cache(...)
        # ... many more types
```

**Characteristics:**
- Factory pattern for cache types
- Each cache type independent
- DualCache for layered caching
- Callback-based cache management

### 6.2 Configuration

#### litellm-rs

```yaml
# config/gateway.yaml
storage:
  database:
    url: "postgresql://localhost/litellm"
    max_connections: 10
    ssl: true
  redis:
    url: "redis://localhost:6379"
    enabled: true
    cluster: false
  vector_db:
    db_type: "qdrant"
    url: "http://localhost:6333"
    api_key: "..."
```

#### litellm (Python)

```python
# Via constructor or litellm.cache
litellm.cache = Cache(
    type="redis-semantic",
    host="localhost",
    port=6379,
    similarity_threshold=0.8,
    embedding_model="text-embedding-ada-002"
)
```

---

## 7. Feature Comparison Matrix

| Feature | litellm-rs | litellm |
|---------|------------|---------|
| **Database** | | |
| PostgreSQL | Yes (SeaORM) | Yes (Prisma) |
| SQLite | Yes (fallback) | No |
| MySQL | No (possible) | No |
| RDS IAM Auth | No | Yes |
| **Caching** | | |
| Redis | Yes | Yes |
| Redis Cluster | No | Yes |
| In-Memory | No (noop mode) | Yes (full-featured) |
| Disk Cache | No | Yes |
| DualCache | No | Yes |
| **Semantic Cache** | | |
| Redis Semantic | No | Yes (RedisVL) |
| Qdrant Semantic | No | Yes |
| **Object Storage** | | |
| Local Files | Yes | Yes (DiskCache) |
| AWS S3 | Yes | Yes |
| Google GCS | No | Yes |
| Azure Blob | No | Yes |
| **Vector DB** | | |
| Qdrant | Yes | Yes (semantic only) |
| Pinecone | Yes | No |
| Weaviate | Yes | No |
| **Performance** | | |
| Connection Pooling | Yes | Yes |
| Batch Operations | Yes | Yes |
| Pipeline Support | Yes | Yes |
| **Resilience** | | |
| Graceful Degradation | Yes (noop mode) | Yes (separate) |
| Health Checks | Yes (unified) | Yes (per-service) |
| Auto-reconnect | SQLite fallback | RDS token refresh |

---

## 8. Recommendations

### For litellm-rs (Rust)

1. **Add Semantic Cache Layer**
   - Implement semantic caching on top of vector stores
   - Add embedding model integration

2. **Expand Cloud Storage**
   - Add GCS and Azure Blob support
   - Consider unified cloud storage trait

3. **Add In-Memory Cache**
   - Implement proper in-memory cache with LRU/TTL
   - Current noop mode is not a true cache

4. **Redis Cluster Support**
   - Add cluster mode for high-availability deployments

### For litellm (Python)

1. **Database Flexibility**
   - Consider SQLite for local development
   - MySQL support for enterprise needs

2. **Unified Storage Interface**
   - Consider adopting a StorageLayer pattern
   - Simplify configuration

3. **Vector DB Expansion**
   - Add Pinecone/Weaviate for semantic caching
   - Support more vector backends

---

## 9. Code References

### litellm-rs (Rust)
- Storage entry point: `/src/storage/mod.rs`
- Database: `/src/storage/database/`
- Redis: `/src/storage/redis/`
- Files: `/src/storage/files/`
- Vector: `/src/storage/vector/`
- Config: `/src/config/models/storage.rs`

### litellm (Python)
- Caching entry point: `/litellm/caching/caching.py`
- Base cache: `/litellm/caching/base_cache.py`
- Redis cache: `/litellm/caching/redis_cache.py`
- Semantic caches: `/litellm/caching/qdrant_semantic_cache.py`, `/litellm/caching/redis_semantic_cache.py`
- Database: `/litellm/proxy/db/prisma_client.py`
- Cloud storage: `/litellm/caching/s3_cache.py`, `/litellm/caching/gcs_cache.py`
