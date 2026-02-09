//! Virtual key request types

use super::types::{Permission, RateLimits};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Virtual key creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKeyRequest {
    /// Key alias (optional)
    pub key_alias: Option<String>,
    /// User ID
    pub user_id: String,
    /// Team ID (optional)
    pub team_id: Option<String>,
    /// Models to allow
    pub models: Vec<String>,
    /// Maximum budget
    pub max_budget: Option<f64>,
    /// Budget duration
    pub budget_duration: Option<String>,
    /// Rate limits
    pub rate_limits: Option<RateLimits>,
    /// Permissions
    pub permissions: Vec<Permission>,
    /// Metadata
    pub metadata: HashMap<String, String>,
    /// Expiration time
    pub expires_at: Option<DateTime<Utc>>,
    /// Tags
    pub tags: Vec<String>,
}

/// Virtual key update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateKeyRequest {
    /// Key alias
    pub key_alias: Option<String>,
    /// Models to allow
    pub models: Option<Vec<String>>,
    /// Maximum budget
    pub max_budget: Option<f64>,
    /// Budget duration
    pub budget_duration: Option<String>,
    /// Rate limits
    pub rate_limits: Option<RateLimits>,
    /// Permissions
    pub permissions: Option<Vec<Permission>>,
    /// Metadata
    pub metadata: Option<HashMap<String, String>>,
    /// Expiration time
    pub expires_at: Option<DateTime<Utc>>,
    /// Whether key is active
    pub is_active: Option<bool>,
    /// Tags
    pub tags: Option<Vec<String>>,
}


#[cfg(test)]
#[path = "requests_tests.rs"]
mod tests;
