# Test Architecture Refactoring Plan

> **ARCHIVED — 2026-05-07.** This plan was largely delivered alongside the audit-remediation campaign. `tests/common/{mod,fixtures,database,providers,assertions}.rs`, `tests/integration/`, and `tests/e2e/` are all in place; `Database::new_mock()` references are gone; the suite is at 10481 lib + 145 integration + 96 doc tests as of [`closeout-2026-05-02.md`](../closeout-2026-05-02.md). Remaining gap is CI-side E2E secret management, not source-tree work. Preserved for cross-reference.

## Executive Summary

基于对现有测试架构的全面分析，本计划旨在重构整个测试系统，消除所有 mock 测试，建立一个基于真实实现的高质量测试架构。

---

## 1. Current State Analysis

### 1.1 Current Test Statistics
- **Total Tests**: 696+ tests
- **Async Tests**: 227 tests
- **Test Modules**: 179+ files containing tests

### 1.2 Identified Issues

#### Problem 1: Mock Database Implementation Missing/Broken
```rust
// These tests reference non-existent Database::new_mock()
// src/core/virtual_keys.rs:587
database: Arc::new(Database::new_mock()),  // ❌ Does not exist

// src/core/user_management.rs:580
let db = Arc::new(Database::new_mock());  // ❌ Does not exist
```

#### Problem 2: Scattered Test Architecture
- Tests are inline in 179+ files
- No centralized test utilities
- No shared fixtures or helpers
- Inconsistent test patterns

#### Problem 3: Mock-Heavy Approach
- Many tests rely on mock data
- Mock implementations are incomplete
- Tests don't verify real behavior

#### Problem 4: Missing Integration Tests
- Only 1 integration test file (`tests/test_connection_pool.rs`)
- No end-to-end API tests
- No real database integration tests

---

## 2. Target Architecture

### 2.1 Test Pyramid

```
          /\
         /  \         E2E Tests (5%)
        /    \        - Full HTTP API tests
       /------\       - Real database + server
      /        \
     /----------\     Integration Tests (25%)
    /            \    - Cross-module tests
   /--------------\   - Real in-memory database
  /                \
 /------------------\ Unit Tests (70%)
                      - Isolated function tests
                      - Real implementations (no mocks)
```

### 2.2 Directory Structure

```
tests/
├── common/
│   ├── mod.rs              # Test utilities module
│   ├── fixtures.rs         # Shared test data
│   ├── database.rs         # Test database setup
│   ├── providers.rs        # Test provider setup
│   └── assertions.rs       # Custom assertions
├── unit/
│   ├── mod.rs
│   ├── providers/          # Provider unit tests
│   ├── router/             # Router unit tests
│   ├── auth/               # Auth unit tests
│   └── config/             # Config unit tests
├── integration/
│   ├── mod.rs
│   ├── api_tests.rs        # API integration tests
│   ├── database_tests.rs   # Database integration tests
│   ├── router_tests.rs     # Router integration tests
│   └── provider_tests.rs   # Provider integration tests
└── e2e/
    ├── mod.rs
    ├── chat_completion.rs  # Chat completion E2E
    ├── embeddings.rs       # Embeddings E2E
    └── batch.rs            # Batch API E2E
```

---

## 3. Implementation Plan

### Phase 1: Test Infrastructure (Core Foundation)

#### 1.1 Create Test Utilities Module
**File**: `tests/common/mod.rs`

```rust
//! Common test utilities for litellm-rs
//!
//! Provides shared infrastructure for all tests:
//! - In-memory SQLite database
//! - Test fixtures and factories
//! - Custom assertions
//! - Provider test utilities

pub mod database;
pub mod fixtures;
pub mod providers;
pub mod assertions;
pub mod server;
```

#### 1.2 In-Memory Database Support
**File**: `tests/common/database.rs`

```rust
use litellm_rs::storage::database::SqliteDatabase;
use litellm_rs::config::DatabaseConfig;

/// Create an in-memory SQLite database for testing
pub async fn create_test_database() -> SqliteDatabase {
    let config = DatabaseConfig {
        url: "sqlite::memory:".to_string(),
        max_connections: 5,
        connection_timeout: 5,
        ssl: false,
        enabled: true,
    };

    let db = SqliteDatabase::new(&config).await
        .expect("Failed to create test database");

    db.migrate().await
        .expect("Failed to run migrations");

    db
}

/// Test database with seeded data
pub async fn create_seeded_database() -> SqliteDatabase {
    let db = create_test_database().await;
    // Seed with test data
    seed_test_data(&db).await;
    db
}
```

#### 1.3 Test Fixtures
**File**: `tests/common/fixtures.rs`

```rust
use litellm_rs::core::types::requests::*;
use litellm_rs::core::models::user::User;
use uuid::Uuid;
use chrono::Utc;

/// User factory for creating test users
pub struct UserFactory;

impl UserFactory {
    pub fn create() -> User {
        User {
            id: Uuid::new_v4(),
            email: format!("test-{}@example.com", Uuid::new_v4()),
            username: format!("user_{}", Uuid::new_v4().to_string()[..8].to_string()),
            display_name: Some("Test User".to_string()),
            ..Default::default()
        }
    }

    pub fn create_admin() -> User {
        let mut user = Self::create();
        user.role = "admin".to_string();
        user
    }
}

/// Chat request factory
pub struct ChatRequestFactory;

impl ChatRequestFactory {
    pub fn simple(model: &str, content: &str) -> ChatRequest {
        ChatRequest {
            model: model.to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text(content.to_string())),
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    pub fn with_system(model: &str, system: &str, user: &str) -> ChatRequest {
        ChatRequest {
            model: model.to_string(),
            messages: vec![
                ChatMessage {
                    role: MessageRole::System,
                    content: Some(MessageContent::Text(system.to_string())),
                    ..Default::default()
                },
                ChatMessage {
                    role: MessageRole::User,
                    content: Some(MessageContent::Text(user.to_string())),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }

    pub fn streaming(model: &str, content: &str) -> ChatRequest {
        let mut request = Self::simple(model, content);
        request.stream = true;
        request
    }
}
```

### Phase 2: Provider Test Infrastructure

#### 2.1 Real Provider Testing
**File**: `tests/common/providers.rs`

```rust
use litellm_rs::core::providers::*;
use std::env;

/// Provider test configuration
pub struct ProviderTestConfig {
    pub skip_live_tests: bool,
}

impl Default for ProviderTestConfig {
    fn default() -> Self {
        Self {
            skip_live_tests: env::var("SKIP_LIVE_TESTS").is_ok(),
        }
    }
}

/// Create a real Groq provider for testing
pub async fn create_groq_provider() -> Option<GroqProvider> {
    let api_key = env::var("GROQ_API_KEY").ok()?;
    Some(GroqProvider::with_api_key(&api_key).await.ok()?)
}

/// Create a real OpenAI provider for testing
pub async fn create_openai_provider() -> Option<OpenAIProvider> {
    let api_key = env::var("OPENAI_API_KEY").ok()?;
    Some(OpenAIProvider::with_api_key(&api_key).await.ok()?)
}

/// Macro for skipping tests that require API keys
#[macro_export]
macro_rules! skip_without_api_key {
    ($var:expr) => {
        if std::env::var($var).is_err() {
            eprintln!("Skipping test: {} not set", $var);
            return;
        }
    };
}
```

### Phase 3: Refactor Existing Tests

#### 3.1 Virtual Keys Tests
**Before**: Uses non-existent `Database::new_mock()`
**After**: Uses real in-memory SQLite

```rust
// tests/unit/virtual_keys.rs
use crate::common::database::create_test_database;

#[tokio::test]
async fn test_key_generation() {
    let db = Arc::new(create_test_database().await);
    let manager = VirtualKeyManager::new(db);

    let key = manager.generate_api_key();
    assert!(key.starts_with("sk-"));
    assert_eq!(key.len(), 35);
}

#[tokio::test]
async fn test_key_validation() {
    let db = Arc::new(create_test_database().await);
    let manager = VirtualKeyManager::new(db);

    // Create and verify a real key
    let key_info = manager.create_key(CreateKeyRequest {
        user_id: "test_user".to_string(),
        ..Default::default()
    }).await.unwrap();

    let validated = manager.validate_key(&key_info.key).await.unwrap();
    assert!(validated.is_valid);
}
```

#### 3.2 User Management Tests
**Before**: Uses non-existent mock
**After**: Real database operations

```rust
// tests/unit/user_management.rs
use crate::common::{database::create_test_database, fixtures::UserFactory};

#[tokio::test]
async fn test_user_creation() {
    let db = Arc::new(create_test_database().await);
    let manager = UserManager::new(db);

    let user = manager.create_user(
        "test@example.com".to_string(),
        Some("Test User".to_string()),
    ).await.unwrap();

    assert_eq!(user.email, "test@example.com");

    // Verify user persisted
    let found = manager.find_by_email("test@example.com").await.unwrap();
    assert!(found.is_some());
}
```

### Phase 4: Integration Tests

#### 4.1 API Integration Tests
**File**: `tests/integration/api_tests.rs`

```rust
use actix_web::{test, App};
use litellm_rs::server::create_app;
use crate::common::database::create_seeded_database;

#[actix_rt::test]
async fn test_health_endpoint() {
    let app = test::init_service(create_app().await).await;

    let req = test::TestRequest::get()
        .uri("/health")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn test_chat_completion_endpoint() {
    let app = test::init_service(create_app().await).await;

    let req = test::TestRequest::post()
        .uri("/v1/chat/completions")
        .set_json(&ChatRequestFactory::simple("gpt-3.5-turbo", "Hello"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Should fail without API key
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
```

#### 4.2 Router Integration Tests
**File**: `tests/integration/router_tests.rs`

```rust
use litellm_rs::core::router::*;
use crate::common::fixtures::ChatRequestFactory;

#[tokio::test]
async fn test_round_robin_routing() {
    let lb = LoadBalancer::new(RoutingStrategy::RoundRobin).await.unwrap();

    // Add multiple deployments
    lb.add_deployment("deployment-1", "openai/gpt-4").await;
    lb.add_deployment("deployment-2", "openai/gpt-4").await;

    let request = ChatRequestFactory::simple("gpt-4", "test");

    // First request should go to deployment-1
    let deployment1 = lb.route(&request).await.unwrap();

    // Second request should go to deployment-2 (round robin)
    let deployment2 = lb.route(&request).await.unwrap();

    assert_ne!(deployment1.id, deployment2.id);
}

#[tokio::test]
async fn test_fallback_routing() {
    let mut config = FallbackConfig::new();
    config.add_context_window_fallback("gpt-4", vec!["gpt-4-32k".to_string()]);

    let lb = LoadBalancer::with_fallbacks(RoutingStrategy::RoundRobin, config)
        .await.unwrap();

    let error = ProviderError::ContextLengthExceeded {
        provider: "openai",
        max: 8192,
        actual: 10000,
    };

    let fallbacks = lb.select_fallback_models(&error, "gpt-4");
    assert_eq!(fallbacks, Some(vec!["gpt-4-32k".to_string()]));
}
```

### Phase 5: E2E Tests (Optional, requires API keys)

#### 5.1 Chat Completion E2E
**File**: `tests/e2e/chat_completion.rs`

```rust
#[tokio::test]
#[ignore] // Run with: cargo test --ignored
async fn test_real_chat_completion() {
    skip_without_api_key!("OPENAI_API_KEY");

    let client = create_gateway_client().await;

    let response = client.chat_completion(
        ChatRequestFactory::simple("gpt-3.5-turbo", "Say hello")
    ).await.unwrap();

    assert!(response.choices.len() > 0);
    assert!(response.choices[0].message.content.is_some());
}

#[tokio::test]
#[ignore]
async fn test_real_streaming() {
    skip_without_api_key!("OPENAI_API_KEY");

    let client = create_gateway_client().await;

    let mut stream = client.chat_completion_stream(
        ChatRequestFactory::streaming("gpt-3.5-turbo", "Count to 5")
    ).await.unwrap();

    let mut chunks = 0;
    while let Some(chunk) = stream.next().await {
        chunks += 1;
        assert!(chunk.is_ok());
    }

    assert!(chunks > 0);
}
```

---

## 4. Test Categories and Markers

### 4.1 Test Markers

```rust
// Unit test - fast, isolated
#[test]
fn test_config_validation() { }

// Async unit test
#[tokio::test]
async fn test_provider_creation() { }

// Integration test - uses real database
#[tokio::test]
async fn test_database_operations() { }

// E2E test - requires external services
#[tokio::test]
#[ignore] // Skip by default
async fn test_real_api_call() { }

// Slow test marker
#[tokio::test]
#[cfg(feature = "slow-tests")]
async fn test_load_balancer_performance() { }
```

### 4.2 Test Execution

```bash
# Run all fast tests (default)
cargo test --all-features

# Run unit tests only
cargo test --lib --all-features

# Run integration tests
cargo test --test '*' --all-features

# Run ignored E2E tests (requires API keys)
cargo test --all-features -- --ignored

# Run with coverage
cargo llvm-cov --all-features

# Run specific test category
cargo test router::tests --all-features
```

---

## 5. Files to Create/Modify

### New Files

| File | Description |
|------|-------------|
| `tests/common/mod.rs` | Test utilities module |
| `tests/common/database.rs` | In-memory database setup |
| `tests/common/fixtures.rs` | Test data factories |
| `tests/common/providers.rs` | Provider test helpers |
| `tests/common/assertions.rs` | Custom assertions |
| `tests/common/server.rs` | Test server utilities |
| `tests/integration/mod.rs` | Integration tests module |
| `tests/integration/api_tests.rs` | API integration tests |
| `tests/integration/router_tests.rs` | Router integration tests |
| `tests/integration/database_tests.rs` | Database integration tests |
| `tests/e2e/mod.rs` | E2E tests module |
| `tests/e2e/chat_completion.rs` | Chat completion E2E |

### Files to Modify

| File | Changes |
|------|---------|
| `src/core/virtual_keys.rs` | Remove mock tests, use real DB |
| `src/core/user_management.rs` | Remove mock tests, use real DB |
| `src/storage/database/seaorm_db.rs` | Add in-memory SQLite support |
| `Cargo.toml` | Add test dependencies |

---

## 6. Implementation Order

### Week 1: Foundation
1. Create `tests/common/` infrastructure
2. Implement in-memory SQLite database support
3. Create basic fixtures and factories

### Week 2: Unit Test Refactoring
4. Refactor `virtual_keys` tests
5. Refactor `user_management` tests
6. Refactor `auth` tests
7. Refactor `config` tests

### Week 3: Integration Tests
8. Create API integration tests
9. Create router integration tests
10. Create database integration tests

### Week 4: Polish and Documentation
11. Add E2E tests (optional)
12. Update documentation
13. Set up CI/CD for tests
14. Final review and cleanup

---

## 7. Success Criteria

- [ ] All 696+ tests pass
- [ ] No mock implementations used
- [ ] Test coverage >= 80%
- [ ] All tests run in < 60 seconds
- [ ] Integration tests use real in-memory database
- [ ] E2E tests available for manual verification
- [ ] CI/CD pipeline configured

---

## 8. Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Breaking existing tests | Incremental migration, keep old tests until new ones pass |
| In-memory DB limitations | Use SQLite for tests, PostgreSQL for production |
| API key exposure | Use environment variables, never commit keys |
| Slow tests | Parallel execution, skip E2E by default |
