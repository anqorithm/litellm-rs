//! API Key Management System
//!
//! This module provides comprehensive API key management functionality including:
//! - Key generation with secure hashing
//! - Key validation and verification
//! - Key rotation and revocation
//! - Permission and rate limit management
//! - Usage tracking and statistics

mod database_repository;
mod db_mapping;
mod db_update;
mod manager;
mod repository;
mod types;

#[cfg(test)]
mod database_repository_tests;
#[cfg(test)]
mod tests;

// Re-export public types
pub use database_repository::DatabaseKeyRepository;
pub use manager::KeyManager;
pub use repository::{InMemoryKeyRepository, KeyRepository};
pub use types::{
    CreateKeyConfig, KeyInfo, KeyPermissions, KeyRateLimits, KeyStatus, KeyUsageStats,
    ManagedApiKey, UpdateKeyConfig, VerifyKeyResult,
};
