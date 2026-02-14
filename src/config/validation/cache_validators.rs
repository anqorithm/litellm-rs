//! Cache and rate limit configuration validators
//!
//! This module provides validation implementations for cache and rate limiting
//! configuration structures including CacheConfig and RateLimitConfig.

use super::trait_def::Validate;
use crate::config::models::cache::CacheConfig;
use crate::core::types::config::rate_limit::RateLimitConfig;

impl Validate for CacheConfig {
    fn validate(&self) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }

        if self.ttl == 0 {
            return Err("Cache TTL must be greater than 0".to_string());
        }

        if self.max_size == 0 {
            return Err("Cache max size must be greater than 0".to_string());
        }

        if self.semantic_cache
            && (self.similarity_threshold <= 0.0 || self.similarity_threshold > 1.0)
        {
            return Err("Semantic cache similarity threshold must be between 0 and 1".to_string());
        }

        Ok(())
    }
}

impl Validate for RateLimitConfig {
    fn validate(&self) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }

        if self.default_rpm == 0 {
            return Err("Default RPM must be greater than 0".to_string());
        }

        if self.default_tpm == 0 {
            return Err("Default TPM must be greater than 0".to_string());
        }

        Ok(())
    }
}
