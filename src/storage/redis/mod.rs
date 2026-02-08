//! Redis storage implementation
//!
//! This module provides Redis connectivity and caching operations.
//!
//! ## Module Structure
//!
//! - `pool` - Connection pool and core connection management
//! - `cache` - Basic cache operations (get, set, delete, exists, expire, ttl)
//! - `batch` - Batch operations (mget, mset)
//! - `collections` - List and Set operations
//! - `hash` - Hash and Sorted Set operations
//! - `pubsub` - Pub/Sub operations (temporarily disabled)
//! - `atomic` - Atomic operations and utilities
//! - `tests` - Module tests

// Module declarations
mod atomic;
mod batch;
mod cache;
mod collections;
mod hash;
mod pool;
mod pubsub;
#[cfg(test)]
mod tests;

// Re-export public types
pub use pool::{RedisConnection, RedisPool};
pub use pubsub::Subscription;
