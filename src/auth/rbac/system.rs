//! RBAC system core functionality

use crate::config::models::auth::RbacConfig;
use crate::utils::error::gateway_error::Result;
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
mod tests {
    use super::*;
    use std::collections::HashSet;

    // ==================== Helper Functions ====================

    fn create_test_config() -> RbacConfig {
        RbacConfig::default()
    }

    fn create_enabled_config() -> RbacConfig {
        RbacConfig {
            enabled: true,
            default_role: "user".to_string(),
            admin_roles: vec!["super_admin".to_string(), "admin".to_string()],
        }
    }

    // ==================== RbacSystem Creation Tests ====================

    #[tokio::test]
    async fn test_rbac_system_creation() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        assert!(!rbac.roles.is_empty());
        assert!(!rbac.permissions.is_empty());
    }

    #[tokio::test]
    async fn test_rbac_system_with_enabled_config() {
        let config = create_enabled_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        assert!(rbac.config.enabled);
        assert_eq!(rbac.config.default_role, "user");
    }

    #[tokio::test]
    async fn test_rbac_system_clone() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();
        let cloned = rbac.clone();

        assert_eq!(cloned.roles.len(), rbac.roles.len());
        assert_eq!(cloned.permissions.len(), rbac.permissions.len());
    }

    #[tokio::test]
    async fn test_rbac_system_debug() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();
        let debug_str = format!("{:?}", rbac);

        assert!(debug_str.contains("RbacSystem"));
        assert!(debug_str.contains("roles"));
        assert!(debug_str.contains("permissions"));
    }

    // ==================== Default Permissions Tests ====================

    #[tokio::test]
    async fn test_default_permissions_initialized() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        // Should have all default permissions
        assert!(rbac.permissions.contains_key("users.read"));
        assert!(rbac.permissions.contains_key("users.write"));
        assert!(rbac.permissions.contains_key("users.delete"));
        assert!(rbac.permissions.contains_key("teams.read"));
        assert!(rbac.permissions.contains_key("teams.write"));
        assert!(rbac.permissions.contains_key("teams.delete"));
        assert!(rbac.permissions.contains_key("api.chat"));
        assert!(rbac.permissions.contains_key("api.embeddings"));
        assert!(rbac.permissions.contains_key("api.images"));
        assert!(rbac.permissions.contains_key("api_keys.read"));
        assert!(rbac.permissions.contains_key("api_keys.write"));
        assert!(rbac.permissions.contains_key("api_keys.delete"));
        assert!(rbac.permissions.contains_key("analytics.read"));
        assert!(rbac.permissions.contains_key("system.admin"));
    }

    #[tokio::test]
    async fn test_permission_count() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        // Should have exactly 14 default permissions
        assert_eq!(rbac.permissions.len(), 14);
    }

    #[tokio::test]
    async fn test_user_permissions_structure() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        let users_read = rbac.permissions.get("users.read").unwrap();
        assert_eq!(users_read.name, "users.read");
        assert_eq!(users_read.resource, "users");
        assert_eq!(users_read.action, "read");
        assert!(users_read.is_system);
    }

    #[tokio::test]
    async fn test_api_permissions_structure() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        let api_chat = rbac.permissions.get("api.chat").unwrap();
        assert_eq!(api_chat.resource, "api");
        assert_eq!(api_chat.action, "chat");
        assert!(api_chat.is_system);
    }

    #[tokio::test]
    async fn test_system_admin_permission() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        let system_admin = rbac.permissions.get("system.admin").unwrap();
        assert_eq!(system_admin.resource, "system");
        assert_eq!(system_admin.action, "admin");
        assert!(system_admin.is_system);
    }

    // ==================== Default Roles Tests ====================

    #[tokio::test]
    async fn test_default_roles_initialized() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        // Should have all default roles
        assert!(rbac.roles.contains_key("super_admin"));
        assert!(rbac.roles.contains_key("admin"));
        assert!(rbac.roles.contains_key("manager"));
        assert!(rbac.roles.contains_key("user"));
        assert!(rbac.roles.contains_key("viewer"));
        assert!(rbac.roles.contains_key("api_user"));
    }

    #[tokio::test]
    async fn test_role_count() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        // Should have exactly 6 default roles
        assert_eq!(rbac.roles.len(), 6);
    }

    #[tokio::test]
    async fn test_super_admin_role() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        let super_admin = rbac.roles.get("super_admin").unwrap();
        assert_eq!(super_admin.name, "super_admin");
        assert!(super_admin.is_system);
        // Super admin should have all permissions
        assert_eq!(super_admin.permissions.len(), rbac.permissions.len());
    }

    #[tokio::test]
    async fn test_admin_role_permissions() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        let admin = rbac.roles.get("admin").unwrap();
        assert!(admin.permissions.contains("users.read"));
        assert!(admin.permissions.contains("users.write"));
        assert!(admin.permissions.contains("api.chat"));
        assert!(admin.permissions.contains("analytics.read"));
        // Admin should not have system.admin or delete permissions
        assert!(!admin.permissions.contains("system.admin"));
        assert!(!admin.permissions.contains("users.delete"));
    }

    #[tokio::test]
    async fn test_manager_role_permissions() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        let manager = rbac.roles.get("manager").unwrap();
        assert!(manager.permissions.contains("teams.read"));
        assert!(manager.permissions.contains("teams.write"));
        assert!(manager.permissions.contains("api.chat"));
        assert!(!manager.permissions.contains("users.write"));
    }

    #[tokio::test]
    async fn test_user_role_permissions() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        let user = rbac.roles.get("user").unwrap();
        assert!(user.permissions.contains("api.chat"));
        assert!(user.permissions.contains("api.embeddings"));
        assert!(user.permissions.contains("api_keys.read"));
        assert_eq!(user.permissions.len(), 3);
    }

    #[tokio::test]
    async fn test_viewer_role_permissions() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        let viewer = rbac.roles.get("viewer").unwrap();
        assert!(viewer.permissions.contains("users.read"));
        assert!(viewer.permissions.contains("teams.read"));
        assert!(viewer.permissions.contains("api_keys.read"));
        assert!(viewer.permissions.contains("analytics.read"));
        // Viewer should not have write permissions
        assert!(!viewer.permissions.contains("users.write"));
        assert!(!viewer.permissions.contains("api.chat"));
    }

    #[tokio::test]
    async fn test_api_user_role_permissions() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        let api_user = rbac.roles.get("api_user").unwrap();
        assert!(api_user.permissions.contains("api.chat"));
        assert!(api_user.permissions.contains("api.embeddings"));
        assert!(api_user.permissions.contains("api.images"));
        assert_eq!(api_user.permissions.len(), 3);
    }

    #[tokio::test]
    async fn test_all_roles_are_system_roles() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        for role in rbac.roles.values() {
            assert!(role.is_system, "Role {} should be a system role", role.name);
        }
    }

    #[tokio::test]
    async fn test_roles_have_no_parent_roles() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        for role in rbac.roles.values() {
            assert!(
                role.parent_roles.is_empty(),
                "Role {} should have no parent roles",
                role.name
            );
        }
    }

    // ==================== List Methods Tests ====================

    #[tokio::test]
    async fn test_list_roles() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        let roles = rbac.list_roles();
        assert_eq!(roles.len(), 6);

        let role_names: HashSet<&str> = roles.iter().map(|r| r.name.as_str()).collect();
        assert!(role_names.contains("super_admin"));
        assert!(role_names.contains("admin"));
        assert!(role_names.contains("manager"));
        assert!(role_names.contains("user"));
        assert!(role_names.contains("viewer"));
        assert!(role_names.contains("api_user"));
    }

    #[tokio::test]
    async fn test_list_permissions() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        let permissions = rbac.list_permissions();
        assert_eq!(permissions.len(), 14);

        let perm_names: HashSet<&str> = permissions.iter().map(|p| p.name.as_str()).collect();
        assert!(perm_names.contains("users.read"));
        assert!(perm_names.contains("api.chat"));
        assert!(perm_names.contains("system.admin"));
    }

    // ==================== Integration Tests ====================

    #[tokio::test]
    async fn test_role_permission_consistency() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        // All role permissions should exist in the permissions map
        for role in rbac.roles.values() {
            for perm in &role.permissions {
                assert!(
                    rbac.permissions.contains_key(perm),
                    "Role {} has unknown permission: {}",
                    role.name,
                    perm
                );
            }
        }
    }

    #[tokio::test]
    async fn test_role_hierarchy_by_permission_count() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        let super_admin = rbac.roles.get("super_admin").unwrap();
        let admin = rbac.roles.get("admin").unwrap();
        let manager = rbac.roles.get("manager").unwrap();
        let user = rbac.roles.get("user").unwrap();
        let viewer = rbac.roles.get("viewer").unwrap();

        // Super admin should have more permissions than admin
        assert!(super_admin.permissions.len() > admin.permissions.len());
        // Admin should have more permissions than manager
        assert!(admin.permissions.len() > manager.permissions.len());
        // User and viewer should have fewer permissions
        assert!(manager.permissions.len() > user.permissions.len());
        assert!(viewer.permissions.len() > user.permissions.len());
    }

    #[tokio::test]
    async fn test_permission_check_simulation() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        let user_role = rbac.roles.get("user").unwrap();

        // Simulate permission check
        let can_chat = user_role.permissions.contains("api.chat");
        let can_delete_users = user_role.permissions.contains("users.delete");

        assert!(can_chat);
        assert!(!can_delete_users);
    }

    #[tokio::test]
    async fn test_all_permissions_are_system_permissions() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        for permission in rbac.permissions.values() {
            assert!(
                permission.is_system,
                "Permission {} should be a system permission",
                permission.name
            );
        }
    }

    #[tokio::test]
    async fn test_resource_coverage() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        let resources: HashSet<&str> = rbac
            .permissions
            .values()
            .map(|p| p.resource.as_str())
            .collect();

        assert!(resources.contains("users"));
        assert!(resources.contains("teams"));
        assert!(resources.contains("api"));
        assert!(resources.contains("api_keys"));
        assert!(resources.contains("analytics"));
        assert!(resources.contains("system"));
    }

    #[tokio::test]
    async fn test_action_coverage() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        let actions: HashSet<&str> = rbac
            .permissions
            .values()
            .map(|p| p.action.as_str())
            .collect();

        assert!(actions.contains("read"));
        assert!(actions.contains("write"));
        assert!(actions.contains("delete"));
        assert!(actions.contains("chat"));
        assert!(actions.contains("embeddings"));
        assert!(actions.contains("images"));
        assert!(actions.contains("admin"));
    }

    #[tokio::test]
    async fn test_multiple_rbac_instances() {
        let config1 = create_test_config();
        let config2 = create_enabled_config();

        let rbac1 = RbacSystem::new(&config1).await.unwrap();
        let rbac2 = RbacSystem::new(&config2).await.unwrap();

        // Both should have the same number of roles and permissions
        assert_eq!(rbac1.roles.len(), rbac2.roles.len());
        assert_eq!(rbac1.permissions.len(), rbac2.permissions.len());

        // But configs should be different
        assert_ne!(rbac1.config.enabled, rbac2.config.enabled);
    }

    #[tokio::test]
    async fn test_find_roles_with_permission() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        let permission = "api.chat";
        let roles_with_perm: Vec<&str> = rbac
            .roles
            .values()
            .filter(|r| r.permissions.contains(permission))
            .map(|r| r.name.as_str())
            .collect();

        assert!(roles_with_perm.contains(&"super_admin"));
        assert!(roles_with_perm.contains(&"admin"));
        assert!(roles_with_perm.contains(&"manager"));
        assert!(roles_with_perm.contains(&"user"));
        assert!(roles_with_perm.contains(&"api_user"));
        assert!(!roles_with_perm.contains(&"viewer"));
    }

    #[tokio::test]
    async fn test_find_permissions_for_resource() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        let users_perms: Vec<&str> = rbac
            .permissions
            .values()
            .filter(|p| p.resource == "users")
            .map(|p| p.name.as_str())
            .collect();

        assert_eq!(users_perms.len(), 3);
        assert!(users_perms.contains(&"users.read"));
        assert!(users_perms.contains(&"users.write"));
        assert!(users_perms.contains(&"users.delete"));
    }

    #[tokio::test]
    async fn test_role_description_not_empty() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        for role in rbac.roles.values() {
            assert!(
                !role.description.is_empty(),
                "Role {} should have a description",
                role.name
            );
        }
    }

    #[tokio::test]
    async fn test_permission_description_not_empty() {
        let config = create_test_config();
        let rbac = RbacSystem::new(&config).await.unwrap();

        for permission in rbac.permissions.values() {
            assert!(
                !permission.description.is_empty(),
                "Permission {} should have a description",
                permission.name
            );
        }
    }
}
