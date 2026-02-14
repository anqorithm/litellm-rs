//! Tests for API key functionality
//!
//! This module contains unit tests for API key management.

#[cfg(test)]
use crate::auth::api_key::types::{ApiKeyVerification, CreateApiKeyRequest};
use crate::core::models::{ApiKey, Metadata, RateLimits, UsageStats};
use chrono::{Duration, Utc};
use uuid::Uuid;

// ==================== CreateApiKeyRequest Tests ====================

#[test]
fn test_create_api_key_request() {
    let request = CreateApiKeyRequest {
        name: "Test Key".to_string(),
        user_id: Some(Uuid::new_v4()),
        team_id: None,
        permissions: vec!["read".to_string(), "write".to_string()],
        rate_limits: None,
        expires_at: None,
    };

    assert_eq!(request.name, "Test Key");
    assert!(request.user_id.is_some());
    assert_eq!(request.permissions.len(), 2);
}

#[test]
fn test_create_api_key_request_with_team() {
    let user_id = Uuid::new_v4();
    let team_id = Uuid::new_v4();

    let request = CreateApiKeyRequest {
        name: "Team Key".to_string(),
        user_id: Some(user_id),
        team_id: Some(team_id),
        permissions: vec!["api.chat".to_string()],
        rate_limits: None,
        expires_at: None,
    };

    assert_eq!(request.team_id, Some(team_id));
    assert_eq!(request.user_id, Some(user_id));
}

#[test]
fn test_create_api_key_request_with_rate_limits() {
    let rate_limits = RateLimits {
        rpm: Some(100),
        tpm: Some(50000),
        rpd: Some(10000),
        tpd: Some(1000000),
        concurrent: Some(10),
    };

    let request = CreateApiKeyRequest {
        name: "Rate Limited Key".to_string(),
        user_id: None,
        team_id: None,
        permissions: vec!["api.chat".to_string()],
        rate_limits: Some(rate_limits.clone()),
        expires_at: None,
    };

    assert!(request.rate_limits.is_some());
    let limits = request.rate_limits.unwrap();
    assert_eq!(limits.rpm, Some(100));
    assert_eq!(limits.tpd, Some(1000000));
}

#[test]
fn test_create_api_key_request_with_expiration() {
    let expires_at = Utc::now() + Duration::days(30);

    let request = CreateApiKeyRequest {
        name: "Expiring Key".to_string(),
        user_id: None,
        team_id: None,
        permissions: vec!["api.chat".to_string()],
        rate_limits: None,
        expires_at: Some(expires_at),
    };

    assert!(request.expires_at.is_some());
    assert!(request.expires_at.unwrap() > Utc::now());
}

#[test]
fn test_create_api_key_request_empty_permissions() {
    let request = CreateApiKeyRequest {
        name: "No Permissions Key".to_string(),
        user_id: None,
        team_id: None,
        permissions: vec![],
        rate_limits: None,
        expires_at: None,
    };

    assert!(request.permissions.is_empty());
}

#[test]
fn test_create_api_key_request_clone() {
    let request = CreateApiKeyRequest {
        name: "Clone Test".to_string(),
        user_id: Some(Uuid::new_v4()),
        team_id: None,
        permissions: vec!["read".to_string()],
        rate_limits: None,
        expires_at: None,
    };

    let cloned = request.clone();
    assert_eq!(request.name, cloned.name);
    assert_eq!(request.user_id, cloned.user_id);
    assert_eq!(request.permissions, cloned.permissions);
}

// ==================== ApiKeyVerification Tests ====================

#[test]
fn test_api_key_verification_valid() {
    let verification = ApiKeyVerification {
        api_key: ApiKey {
            metadata: Metadata::new(),
            name: "Test Key".to_string(),
            key_hash: "hash".to_string(),
            key_prefix: "gw-test".to_string(),
            user_id: None,
            team_id: None,
            permissions: vec!["read".to_string()],
            rate_limits: None,
            expires_at: None,
            is_active: true,
            last_used_at: None,
            usage_stats: UsageStats::default(),
        },
        user: None,
        is_valid: true,
        invalid_reason: None,
    };

    assert!(verification.is_valid);
    assert!(verification.invalid_reason.is_none());
    assert_eq!(verification.api_key.name, "Test Key");
}

#[test]
fn test_api_key_verification_invalid_inactive() {
    let verification = ApiKeyVerification {
        api_key: ApiKey {
            metadata: Metadata::new(),
            name: "Inactive Key".to_string(),
            key_hash: "hash".to_string(),
            key_prefix: "gw-test".to_string(),
            user_id: None,
            team_id: None,
            permissions: vec![],
            rate_limits: None,
            expires_at: None,
            is_active: false,
            last_used_at: None,
            usage_stats: UsageStats::default(),
        },
        user: None,
        is_valid: false,
        invalid_reason: Some("API key is inactive".to_string()),
    };

    assert!(!verification.is_valid);
    assert!(verification.invalid_reason.is_some());
    assert_eq!(verification.invalid_reason.unwrap(), "API key is inactive");
}

#[test]
fn test_api_key_verification_invalid_expired() {
    let expired_at = Utc::now() - Duration::days(1);

    let verification = ApiKeyVerification {
        api_key: ApiKey {
            metadata: Metadata::new(),
            name: "Expired Key".to_string(),
            key_hash: "hash".to_string(),
            key_prefix: "gw-test".to_string(),
            user_id: None,
            team_id: None,
            permissions: vec![],
            rate_limits: None,
            expires_at: Some(expired_at),
            is_active: true,
            last_used_at: None,
            usage_stats: UsageStats::default(),
        },
        user: None,
        is_valid: false,
        invalid_reason: Some("API key is expired".to_string()),
    };

    assert!(!verification.is_valid);
    assert!(verification.api_key.expires_at.unwrap() < Utc::now());
}

#[test]
fn test_api_key_verification_not_found() {
    let verification = ApiKeyVerification {
        api_key: ApiKey {
            metadata: Metadata::new(),
            name: "".to_string(),
            key_hash: "".to_string(),
            key_prefix: "".to_string(),
            user_id: None,
            team_id: None,
            permissions: vec![],
            rate_limits: None,
            expires_at: None,
            is_active: false,
            last_used_at: None,
            usage_stats: UsageStats::default(),
        },
        user: None,
        is_valid: false,
        invalid_reason: Some("API key not found".to_string()),
    };

    assert!(!verification.is_valid);
    assert_eq!(verification.invalid_reason.unwrap(), "API key not found");
}

#[test]
fn test_api_key_verification_clone() {
    let verification = ApiKeyVerification {
        api_key: ApiKey {
            metadata: Metadata::new(),
            name: "Clone Test".to_string(),
            key_hash: "hash".to_string(),
            key_prefix: "gw-test".to_string(),
            user_id: None,
            team_id: None,
            permissions: vec!["read".to_string()],
            rate_limits: None,
            expires_at: None,
            is_active: true,
            last_used_at: None,
            usage_stats: UsageStats::default(),
        },
        user: None,
        is_valid: true,
        invalid_reason: None,
    };

    let cloned = verification.clone();
    assert_eq!(verification.is_valid, cloned.is_valid);
    assert_eq!(verification.api_key.name, cloned.api_key.name);
}

// ==================== ApiKey Model Tests ====================

#[test]
fn test_api_key_creation() {
    let api_key = ApiKey {
        metadata: Metadata::new(),
        name: "Test Key".to_string(),
        key_hash: "hash123".to_string(),
        key_prefix: "gw-abcd".to_string(),
        user_id: None,
        team_id: None,
        permissions: vec!["read".to_string(), "write".to_string()],
        rate_limits: None,
        expires_at: None,
        is_active: true,
        last_used_at: None,
        usage_stats: UsageStats::default(),
    };

    assert_eq!(api_key.name, "Test Key");
    assert!(api_key.is_active);
    assert_eq!(api_key.permissions.len(), 2);
}

#[test]
fn test_api_key_with_user_and_team() {
    let user_id = Uuid::new_v4();
    let team_id = Uuid::new_v4();

    let api_key = ApiKey {
        metadata: Metadata::new(),
        name: "User Team Key".to_string(),
        key_hash: "hash".to_string(),
        key_prefix: "gw-test".to_string(),
        user_id: Some(user_id),
        team_id: Some(team_id),
        permissions: vec!["api.chat".to_string()],
        rate_limits: None,
        expires_at: None,
        is_active: true,
        last_used_at: None,
        usage_stats: UsageStats::default(),
    };

    assert_eq!(api_key.user_id, Some(user_id));
    assert_eq!(api_key.team_id, Some(team_id));
}

#[test]
fn test_api_key_with_rate_limits() {
    let rate_limits = RateLimits {
        rpm: Some(60),
        tpm: Some(100000),
        rpd: Some(5000),
        tpd: Some(500000),
        concurrent: Some(5),
    };

    let api_key = ApiKey {
        metadata: Metadata::new(),
        name: "Rate Limited".to_string(),
        key_hash: "hash".to_string(),
        key_prefix: "gw-test".to_string(),
        user_id: None,
        team_id: None,
        permissions: vec![],
        rate_limits: Some(rate_limits),
        expires_at: None,
        is_active: true,
        last_used_at: None,
        usage_stats: UsageStats::default(),
    };

    assert!(api_key.rate_limits.is_some());
    let limits = api_key.rate_limits.unwrap();
    assert_eq!(limits.rpm, Some(60));
    assert_eq!(limits.concurrent, Some(5));
}

#[test]
fn test_api_key_with_last_used() {
    let last_used = Utc::now() - Duration::hours(1);

    let api_key = ApiKey {
        metadata: Metadata::new(),
        name: "Recently Used".to_string(),
        key_hash: "hash".to_string(),
        key_prefix: "gw-test".to_string(),
        user_id: None,
        team_id: None,
        permissions: vec![],
        rate_limits: None,
        expires_at: None,
        is_active: true,
        last_used_at: Some(last_used),
        usage_stats: UsageStats::default(),
    };

    assert!(api_key.last_used_at.is_some());
    assert!(api_key.last_used_at.unwrap() < Utc::now());
}

#[test]
fn test_api_key_permissions_check() {
    let api_key = ApiKey {
        metadata: Metadata::new(),
        name: "Permission Test".to_string(),
        key_hash: "hash".to_string(),
        key_prefix: "gw-test".to_string(),
        user_id: None,
        team_id: None,
        permissions: vec![
            "api.chat".to_string(),
            "api.embeddings".to_string(),
            "api.images".to_string(),
        ],
        rate_limits: None,
        expires_at: None,
        is_active: true,
        last_used_at: None,
        usage_stats: UsageStats::default(),
    };

    assert!(api_key.permissions.contains(&"api.chat".to_string()));
    assert!(api_key.permissions.contains(&"api.embeddings".to_string()));
    assert!(api_key.permissions.contains(&"api.images".to_string()));
    assert!(!api_key.permissions.contains(&"admin".to_string()));
}

// ==================== UsageStats Tests ====================

#[test]
fn test_usage_stats_default() {
    let stats = UsageStats::default();
    assert_eq!(stats.total_requests, 0);
    assert_eq!(stats.total_tokens, 0);
    assert_eq!(stats.total_cost, 0.0);
    assert_eq!(stats.requests_today, 0);
    assert_eq!(stats.tokens_today, 0);
    assert_eq!(stats.cost_today, 0.0);
}

#[test]
fn test_usage_stats_with_values() {
    let stats = UsageStats {
        total_requests: 1000,
        total_tokens: 500000,
        total_cost: 25.50,
        requests_today: 100,
        tokens_today: 50000,
        cost_today: 2.50,
        last_reset: Utc::now(),
    };

    assert_eq!(stats.total_requests, 1000);
    assert_eq!(stats.total_tokens, 500000);
    assert!((stats.total_cost - 25.50).abs() < f64::EPSILON);
    assert_eq!(stats.requests_today, 100);
}

// ==================== RateLimits Tests ====================

#[test]
fn test_rate_limits_all_set() {
    let limits = RateLimits {
        rpm: Some(100),
        tpm: Some(50000),
        rpd: Some(10000),
        tpd: Some(1000000),
        concurrent: Some(10),
    };

    assert_eq!(limits.rpm, Some(100));
    assert_eq!(limits.rpd, Some(10000));
    assert_eq!(limits.tpm, Some(50000));
    assert_eq!(limits.tpd, Some(1000000));
    assert_eq!(limits.concurrent, Some(10));
}

#[test]
fn test_rate_limits_partial() {
    let limits = RateLimits {
        rpm: Some(60),
        tpm: None,
        rpd: None,
        tpd: Some(100000),
        concurrent: None,
    };

    assert!(limits.rpm.is_some());
    assert!(limits.rpd.is_none());
    assert!(limits.tpd.is_some());
}

#[test]
fn test_rate_limits_clone() {
    let limits = RateLimits {
        rpm: Some(100),
        tpm: Some(50000),
        rpd: Some(10000),
        tpd: Some(1000000),
        concurrent: Some(10),
    };

    let cloned = limits.clone();
    assert_eq!(limits.rpm, cloned.rpm);
    assert_eq!(limits.tpd, cloned.tpd);
}

// ==================== Metadata Tests ====================

#[test]
fn test_metadata_new() {
    let metadata = Metadata::new();
    assert!(!metadata.id.is_nil());
    assert!(metadata.created_at <= Utc::now());
    assert!(metadata.updated_at <= Utc::now());
}

#[test]
fn test_metadata_timestamps() {
    let metadata = Metadata::new();
    // Created and updated should be approximately equal for new metadata
    let diff = metadata.updated_at - metadata.created_at;
    assert!(diff.num_seconds() < 1);
}

// ==================== Key Prefix Tests ====================

#[test]
fn test_key_prefix_format() {
    let api_key = ApiKey {
        metadata: Metadata::new(),
        name: "Prefix Test".to_string(),
        key_hash: "hash".to_string(),
        key_prefix: "gw-abcd1234".to_string(),
        user_id: None,
        team_id: None,
        permissions: vec![],
        rate_limits: None,
        expires_at: None,
        is_active: true,
        last_used_at: None,
        usage_stats: UsageStats::default(),
    };

    assert!(api_key.key_prefix.starts_with("gw-"));
}

#[test]
fn test_key_prefix_different_formats() {
    // Test various valid prefix formats
    let prefixes = vec!["gw-test", "gw-abc123", "gw-UPPER", "gw-mixed123ABC"];

    for prefix in prefixes {
        let api_key = ApiKey {
            metadata: Metadata::new(),
            name: "Test".to_string(),
            key_hash: "hash".to_string(),
            key_prefix: prefix.to_string(),
            user_id: None,
            team_id: None,
            permissions: vec![],
            rate_limits: None,
            expires_at: None,
            is_active: true,
            last_used_at: None,
            usage_stats: UsageStats::default(),
        };

        assert_eq!(api_key.key_prefix, prefix);
    }
}
