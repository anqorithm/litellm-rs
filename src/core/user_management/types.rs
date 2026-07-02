//! Core entity types for user management

use super::roles::{TeamRole, UserRole};
use super::settings::{OrganizationSettings, TeamSettings, UserPreferences};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user identifier
    pub user_id: String,
    /// User email
    pub email: String,
    /// User display name
    pub display_name: Option<String>,
    /// First name
    pub first_name: Option<String>,
    /// Last name
    pub last_name: Option<String>,
    /// User role
    pub role: UserRole,
    /// Teams the user belongs to
    pub teams: Vec<String>,
    /// User permissions
    pub permissions: Vec<String>,
    /// User metadata
    pub metadata: HashMap<String, String>,
    /// Maximum budget for the user
    pub max_budget: Option<f64>,
    /// Current spend
    pub spend: f64,
    /// Budget duration
    pub budget_duration: Option<String>,
    /// Budget reset timestamp
    pub budget_reset_at: Option<DateTime<Utc>>,
    /// Whether user is active
    pub is_active: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last login timestamp
    pub last_login_at: Option<DateTime<Utc>>,
    /// User preferences
    pub preferences: UserPreferences,
}

/// Team entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    /// Unique team identifier
    pub team_id: String,
    /// Team name
    pub team_name: String,
    /// Team description
    pub description: Option<String>,
    /// Organization ID
    pub organization_id: Option<String>,
    /// Team members
    pub members: Vec<TeamMember>,
    /// Team permissions
    pub permissions: Vec<String>,
    /// Models the team can access
    pub models: Vec<String>,
    /// Maximum budget for the team
    pub max_budget: Option<f64>,
    /// Current spend
    pub spend: f64,
    /// Budget duration
    pub budget_duration: Option<String>,
    /// Budget reset timestamp
    pub budget_reset_at: Option<DateTime<Utc>>,
    /// Team metadata
    pub metadata: HashMap<String, String>,
    /// Whether team is active
    pub is_active: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Team settings
    pub settings: TeamSettings,
}

/// Organization entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    /// Unique organization identifier
    pub organization_id: String,
    /// Organization name
    pub organization_name: String,
    /// Organization description
    pub description: Option<String>,
    /// Organization domain
    pub domain: Option<String>,
    /// Teams in the organization
    pub teams: Vec<String>,
    /// Organization admins
    pub admins: Vec<String>,
    /// Maximum budget for the organization
    pub max_budget: Option<f64>,
    /// Current spend
    pub spend: f64,
    /// Budget duration
    pub budget_duration: Option<String>,
    /// Budget reset timestamp
    pub budget_reset_at: Option<DateTime<Utc>>,
    /// Organization metadata
    pub metadata: HashMap<String, String>,
    /// Whether organization is active
    pub is_active: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Organization settings
    pub settings: OrganizationSettings,
}

/// Team member with role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    /// User ID
    pub user_id: String,
    /// Role in the team
    pub role: TeamRole,
    /// When the user joined the team
    pub joined_at: DateTime<Utc>,
    /// Whether the member is active
    pub is_active: bool,
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
