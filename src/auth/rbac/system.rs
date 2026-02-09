//! RBAC system core functionality

use crate::config::models::auth::RbacConfig;
use crate::utils::error::error::Result;
use std::collections::HashMap;
use tracing::{debug, info};

use super::types::{Permission, Role};

/// RBAC system for managing roles and permissions
#[derive(Debug, Clone)]
pub struct RbacSystem {
    /// RBAC configuration
    pub(super) config: RbacConfig,
    /// Role definitions
    pub(super) roles: HashMap<String, Role>,
    /// Permission definitions
    pub(super) permissions: HashMap<String, Permission>,
}

impl RbacSystem {
    /// Create a new RBAC system
    pub async fn new(config: &RbacConfig) -> Result<Self> {
        info!("Initializing RBAC system");

        let mut rbac = Self {
            config: config.clone(),
            roles: HashMap::new(),
            permissions: HashMap::new(),
        };

        // Initialize default permissions and roles
        rbac.initialize_default_permissions().await?;
        rbac.initialize_default_roles().await?;

        info!("RBAC system initialized successfully");
        Ok(rbac)
    }

    /// Initialize default permissions
    pub(super) async fn initialize_default_permissions(&mut self) -> Result<()> {
        debug!("Initializing default permissions");

        let default_permissions = vec![
            // User management
            Permission {
                name: "users.read".to_string(),
                description: "Read user information".to_string(),
                resource: "users".to_string(),
                action: "read".to_string(),
                is_system: true,
            },
            Permission {
                name: "users.write".to_string(),
                description: "Create and update users".to_string(),
                resource: "users".to_string(),
                action: "write".to_string(),
                is_system: true,
            },
            Permission {
                name: "users.delete".to_string(),
                description: "Delete users".to_string(),
                resource: "users".to_string(),
                action: "delete".to_string(),
                is_system: true,
            },
            // Team management
            Permission {
                name: "teams.read".to_string(),
                description: "Read team information".to_string(),
                resource: "teams".to_string(),
                action: "read".to_string(),
                is_system: true,
            },
            Permission {
                name: "teams.write".to_string(),
                description: "Create and update teams".to_string(),
                resource: "teams".to_string(),
                action: "write".to_string(),
                is_system: true,
            },
            Permission {
                name: "teams.delete".to_string(),
                description: "Delete teams".to_string(),
                resource: "teams".to_string(),
                action: "delete".to_string(),
                is_system: true,
            },
            // API access
            Permission {
                name: "api.chat".to_string(),
                description: "Access chat completion API".to_string(),
                resource: "api".to_string(),
                action: "chat".to_string(),
                is_system: true,
            },
            Permission {
                name: "api.embeddings".to_string(),
                description: "Access embeddings API".to_string(),
                resource: "api".to_string(),
                action: "embeddings".to_string(),
                is_system: true,
            },
            Permission {
                name: "api.images".to_string(),
                description: "Access image generation API".to_string(),
                resource: "api".to_string(),
                action: "images".to_string(),
                is_system: true,
            },
            // API key management
            Permission {
                name: "api_keys.read".to_string(),
                description: "Read API key information".to_string(),
                resource: "api_keys".to_string(),
                action: "read".to_string(),
                is_system: true,
            },
            Permission {
                name: "api_keys.write".to_string(),
                description: "Create and update API keys".to_string(),
                resource: "api_keys".to_string(),
                action: "write".to_string(),
                is_system: true,
            },
            Permission {
                name: "api_keys.delete".to_string(),
                description: "Delete API keys".to_string(),
                resource: "api_keys".to_string(),
                action: "delete".to_string(),
                is_system: true,
            },
            // Analytics and monitoring
            Permission {
                name: "analytics.read".to_string(),
                description: "Read analytics and usage data".to_string(),
                resource: "analytics".to_string(),
                action: "read".to_string(),
                is_system: true,
            },
            Permission {
                name: "system.admin".to_string(),
                description: "Full system administration access".to_string(),
                resource: "system".to_string(),
                action: "admin".to_string(),
                is_system: true,
            },
        ];

        for permission in default_permissions {
            self.permissions.insert(permission.name.clone(), permission);
        }

        debug!("Initialized {} default permissions", self.permissions.len());
        Ok(())
    }

    /// Initialize default roles
    pub(super) async fn initialize_default_roles(&mut self) -> Result<()> {
        debug!("Initializing default roles");

        let default_roles = vec![
            // Super Admin - full access
            Role {
                name: "super_admin".to_string(),
                description: "Super administrator with full system access".to_string(),
                permissions: self.permissions.keys().cloned().collect(),
                parent_roles: std::collections::HashSet::new(),
                is_system: true,
            },
            // Admin - most access except super admin functions
            Role {
                name: "admin".to_string(),
                description: "Administrator with broad system access".to_string(),
                permissions: [
                    "users.read",
                    "users.write",
                    "teams.read",
                    "teams.write",
                    "api.chat",
                    "api.embeddings",
                    "api.images",
                    "api_keys.read",
                    "api_keys.write",
                    "api_keys.delete",
                    "analytics.read",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                parent_roles: std::collections::HashSet::new(),
                is_system: true,
            },
            // Manager - team management and API access
            Role {
                name: "manager".to_string(),
                description: "Team manager with API access and team management".to_string(),
                permissions: [
                    "teams.read",
                    "teams.write",
                    "api.chat",
                    "api.embeddings",
                    "api.images",
                    "api_keys.read",
                    "api_keys.write",
                    "analytics.read",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                parent_roles: std::collections::HashSet::new(),
                is_system: true,
            },
            // User - basic API access
            Role {
                name: "user".to_string(),
                description: "Regular user with API access".to_string(),
                permissions: ["api.chat", "api.embeddings", "api_keys.read"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                parent_roles: std::collections::HashSet::new(),
                is_system: true,
            },
            // Viewer - read-only access
            Role {
                name: "viewer".to_string(),
                description: "Read-only access to resources".to_string(),
                permissions: [
                    "users.read",
                    "teams.read",
                    "api_keys.read",
                    "analytics.read",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                parent_roles: std::collections::HashSet::new(),
                is_system: true,
            },
            // API User - API access only
            Role {
                name: "api_user".to_string(),
                description: "API-only access for programmatic use".to_string(),
                permissions: ["api.chat", "api.embeddings", "api.images"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                parent_roles: std::collections::HashSet::new(),
                is_system: true,
            },
        ];

        for role in default_roles {
            self.roles.insert(role.name.clone(), role);
        }

        debug!("Initialized {} default roles", self.roles.len());
        Ok(())
    }

    /// List all roles
    pub fn list_roles(&self) -> Vec<&Role> {
        self.roles.values().collect()
    }

    /// List all permissions
    pub fn list_permissions(&self) -> Vec<&Permission> {
        self.permissions.values().collect()
    }
}


#[cfg(test)]
#[path = "system_tests.rs"]
mod tests;
