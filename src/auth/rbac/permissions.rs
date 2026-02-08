//! Permission checking methods

use crate::core::models::user::types::User;
use crate::utils::error::error::{GatewayError, Result};
use std::collections::HashSet;

use super::helpers::RbacHelpers;
use super::system::RbacSystem;
use super::types::{Permission, PermissionCheck};

impl RbacSystem {
    /// Get all permissions for a user
    pub async fn get_user_permissions(&self, user: &User) -> Result<Vec<String>> {
        let mut permissions = HashSet::new();

        // Get permissions from user role
        let role_name = self.user_role_to_string(&user.role);
        if let Some(role) = self.roles.get(&role_name) {
            permissions.extend(self.get_role_permissions(role));
        }

        Ok(permissions.into_iter().collect())
    }

    /// Check if user has specific permissions
    pub fn check_permissions(
        &self,
        user_permissions: &[String],
        required_permissions: &[String],
    ) -> bool {
        let user_perms: HashSet<&String> = user_permissions.iter().collect();

        // Check for wildcard permission
        if user_perms
            .iter()
            .any(|p| p.as_str() == "*" || p.as_str() == "system.admin")
        {
            return true;
        }

        // Check if user has all required permissions
        required_permissions
            .iter()
            .all(|perm| user_perms.contains(&perm))
    }

    /// Check if user has any of the required permissions
    pub fn check_any_permission(
        &self,
        user_permissions: &[String],
        required_permissions: &[String],
    ) -> bool {
        let user_perms: HashSet<&String> = user_permissions.iter().collect();

        // Check for wildcard permission
        if user_perms
            .iter()
            .any(|p| p.as_str() == "*" || p.as_str() == "system.admin")
        {
            return true;
        }

        // Check if user has any of the required permissions
        required_permissions
            .iter()
            .any(|perm| user_perms.contains(&perm))
    }

    /// Detailed permission check
    pub async fn check_permission_detailed(
        &self,
        user: &User,
        required_permission: &str,
    ) -> Result<PermissionCheck> {
        let user_permissions = self.get_user_permissions(user).await?;
        let user_perms: HashSet<&String> = user_permissions.iter().collect();

        // Check for wildcard or admin permission
        if user_perms
            .iter()
            .any(|p| p.as_str() == "*" || p.as_str() == "system.admin")
        {
            return Ok(PermissionCheck {
                granted: true,
                granted_by_roles: vec![self.user_role_to_string(&user.role)],
                denial_reason: None,
            });
        }

        // Check specific permission
        if user_perms.iter().any(|p| p.as_str() == required_permission) {
            Ok(PermissionCheck {
                granted: true,
                granted_by_roles: vec![self.user_role_to_string(&user.role)],
                denial_reason: None,
            })
        } else {
            Ok(PermissionCheck {
                granted: false,
                granted_by_roles: vec![],
                denial_reason: Some(format!("Missing permission: {}", required_permission)),
            })
        }
    }

    /// Check resource-level permissions
    pub fn check_resource_permission(
        &self,
        user_permissions: &[String],
        resource: &str,
        action: &str,
    ) -> bool {
        let required_permission = format!("{}.{}", resource, action);
        self.check_permissions(user_permissions, &[required_permission])
    }

    /// Check if user is admin
    pub fn is_admin(&self, user: &User) -> bool {
        let role_name = self.user_role_to_string(&user.role);
        self.config.admin_roles.contains(&role_name)
    }

    /// Get permission by name
    pub fn get_permission(&self, permission_name: &str) -> Option<&Permission> {
        self.permissions.get(permission_name)
    }

    /// Add custom permission
    pub fn add_permission(&mut self, permission: Permission) -> Result<()> {
        if permission.is_system {
            return Err(GatewayError::auth("Cannot modify system permissions"));
        }

        self.permissions.insert(permission.name.clone(), permission);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RbacConfig;
    use crate::core::models::user::preferences::UserPreferences;
    use crate::core::models::user::types::{UserProfile, UserRole, UserStatus};
    use crate::core::models::{Metadata, UsageStats};

    // ==================== Helper Functions ====================

    async fn create_test_rbac_system() -> RbacSystem {
        let config = RbacConfig::default();
        RbacSystem::new(&config).await.unwrap()
    }

    async fn create_enabled_rbac_system() -> RbacSystem {
        let config = RbacConfig {
            enabled: true,
            default_role: "user".to_string(),
            admin_roles: vec!["super_admin".to_string(), "admin".to_string()],
        };
        RbacSystem::new(&config).await.unwrap()
    }

    fn create_test_user(role: UserRole) -> User {
        User {
            metadata: Metadata::new(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            display_name: Some("Test User".to_string()),
            password_hash: "hash".to_string(),
            role,
            status: UserStatus::Active,
            team_ids: vec![],
            preferences: UserPreferences::default(),
            usage_stats: UsageStats::default(),
            rate_limits: None,
            last_login_at: None,
            email_verified: true,
            two_factor_enabled: false,
            profile: UserProfile::default(),
        }
    }

    // ==================== get_user_permissions Tests ====================

    #[tokio::test]
    async fn test_get_user_permissions_super_admin() {
        let rbac = create_test_rbac_system().await;
        let user = create_test_user(UserRole::SuperAdmin);

        let perms = rbac.get_user_permissions(&user).await.unwrap();

        // Super admin should have all permissions
        assert!(perms.contains(&"users.read".to_string()));
        assert!(perms.contains(&"users.write".to_string()));
        assert!(perms.contains(&"system.admin".to_string()));
    }

    #[tokio::test]
    async fn test_get_user_permissions_admin() {
        let rbac = create_test_rbac_system().await;
        let user = create_test_user(UserRole::Admin);

        let perms = rbac.get_user_permissions(&user).await.unwrap();

        assert!(perms.contains(&"users.read".to_string()));
        assert!(perms.contains(&"api.chat".to_string()));
        assert!(!perms.contains(&"system.admin".to_string()));
    }

    #[tokio::test]
    async fn test_get_user_permissions_user() {
        let rbac = create_test_rbac_system().await;
        let user = create_test_user(UserRole::User);

        let perms = rbac.get_user_permissions(&user).await.unwrap();

        assert!(perms.contains(&"api.chat".to_string()));
        assert!(perms.contains(&"api.embeddings".to_string()));
        assert_eq!(perms.len(), 3);
    }

    #[tokio::test]
    async fn test_get_user_permissions_viewer() {
        let rbac = create_test_rbac_system().await;
        let user = create_test_user(UserRole::Viewer);

        let perms = rbac.get_user_permissions(&user).await.unwrap();

        assert!(perms.contains(&"users.read".to_string()));
        assert!(perms.contains(&"teams.read".to_string()));
        assert!(!perms.contains(&"api.chat".to_string()));
    }

    #[tokio::test]
    async fn test_get_user_permissions_api_user() {
        let rbac = create_test_rbac_system().await;
        let user = create_test_user(UserRole::ApiUser);

        let perms = rbac.get_user_permissions(&user).await.unwrap();

        assert!(perms.contains(&"api.chat".to_string()));
        assert!(perms.contains(&"api.embeddings".to_string()));
        assert!(perms.contains(&"api.images".to_string()));
        assert_eq!(perms.len(), 3);
    }

    // ==================== check_permissions Tests ====================

    #[tokio::test]
    async fn test_check_permissions_all_present() {
        let rbac = create_test_rbac_system().await;
        let user_perms = vec!["api.chat".to_string(), "api.embeddings".to_string()];
        let required = vec!["api.chat".to_string()];

        let result = rbac.check_permissions(&user_perms, &required);
        assert!(result);
    }

    #[tokio::test]
    async fn test_check_permissions_missing() {
        let rbac = create_test_rbac_system().await;
        let user_perms = vec!["api.chat".to_string()];
        let required = vec!["api.chat".to_string(), "users.write".to_string()];

        let result = rbac.check_permissions(&user_perms, &required);
        assert!(!result);
    }

    #[tokio::test]
    async fn test_check_permissions_empty_required() {
        let rbac = create_test_rbac_system().await;
        let user_perms = vec!["api.chat".to_string()];
        let required: Vec<String> = vec![];

        let result = rbac.check_permissions(&user_perms, &required);
        assert!(result);
    }

    #[tokio::test]
    async fn test_check_permissions_empty_user_perms() {
        let rbac = create_test_rbac_system().await;
        let user_perms: Vec<String> = vec![];
        let required = vec!["api.chat".to_string()];

        let result = rbac.check_permissions(&user_perms, &required);
        assert!(!result);
    }

    #[tokio::test]
    async fn test_check_permissions_wildcard() {
        let rbac = create_test_rbac_system().await;
        let user_perms = vec!["*".to_string()];
        let required = vec!["any.permission".to_string(), "another.one".to_string()];

        let result = rbac.check_permissions(&user_perms, &required);
        assert!(result);
    }

    #[tokio::test]
    async fn test_check_permissions_system_admin() {
        let rbac = create_test_rbac_system().await;
        let user_perms = vec!["system.admin".to_string()];
        let required = vec!["any.permission".to_string()];

        let result = rbac.check_permissions(&user_perms, &required);
        assert!(result);
    }

    #[tokio::test]
    async fn test_check_permissions_exact_match() {
        let rbac = create_test_rbac_system().await;
        let user_perms = vec![
            "api.chat".to_string(),
            "api.embeddings".to_string(),
            "api.images".to_string(),
        ];
        let required = vec![
            "api.chat".to_string(),
            "api.embeddings".to_string(),
            "api.images".to_string(),
        ];

        let result = rbac.check_permissions(&user_perms, &required);
        assert!(result);
    }

    // ==================== check_any_permission Tests ====================

    #[tokio::test]
    async fn test_check_any_permission_one_match() {
        let rbac = create_test_rbac_system().await;
        let user_perms = vec!["api.chat".to_string()];
        let required = vec!["api.chat".to_string(), "users.write".to_string()];

        let result = rbac.check_any_permission(&user_perms, &required);
        assert!(result);
    }

    #[tokio::test]
    async fn test_check_any_permission_no_match() {
        let rbac = create_test_rbac_system().await;
        let user_perms = vec!["api.chat".to_string()];
        let required = vec!["users.write".to_string(), "teams.delete".to_string()];

        let result = rbac.check_any_permission(&user_perms, &required);
        assert!(!result);
    }

    #[tokio::test]
    async fn test_check_any_permission_wildcard() {
        let rbac = create_test_rbac_system().await;
        let user_perms = vec!["*".to_string()];
        let required = vec!["any.permission".to_string()];

        let result = rbac.check_any_permission(&user_perms, &required);
        assert!(result);
    }

    #[tokio::test]
    async fn test_check_any_permission_system_admin() {
        let rbac = create_test_rbac_system().await;
        let user_perms = vec!["system.admin".to_string()];
        let required = vec!["any.permission".to_string()];

        let result = rbac.check_any_permission(&user_perms, &required);
        assert!(result);
    }

    #[tokio::test]
    async fn test_check_any_permission_all_match() {
        let rbac = create_test_rbac_system().await;
        let user_perms = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let required = vec!["a".to_string(), "b".to_string()];

        let result = rbac.check_any_permission(&user_perms, &required);
        assert!(result);
    }

    // ==================== check_permission_detailed Tests ====================

    #[tokio::test]
    async fn test_check_permission_detailed_granted() {
        let rbac = create_test_rbac_system().await;
        let user = create_test_user(UserRole::User);

        let result = rbac
            .check_permission_detailed(&user, "api.chat")
            .await
            .unwrap();

        assert!(result.granted);
        assert!(!result.granted_by_roles.is_empty());
        assert!(result.denial_reason.is_none());
    }

    #[tokio::test]
    async fn test_check_permission_detailed_denied() {
        let rbac = create_test_rbac_system().await;
        let user = create_test_user(UserRole::User);

        let result = rbac
            .check_permission_detailed(&user, "users.delete")
            .await
            .unwrap();

        assert!(!result.granted);
        assert!(result.granted_by_roles.is_empty());
        assert!(result.denial_reason.is_some());
        assert!(result.denial_reason.unwrap().contains("Missing permission"));
    }

    #[tokio::test]
    async fn test_check_permission_detailed_super_admin() {
        let rbac = create_test_rbac_system().await;
        let user = create_test_user(UserRole::SuperAdmin);

        let result = rbac
            .check_permission_detailed(&user, "any.permission")
            .await
            .unwrap();

        // Super admin has system.admin, so should be granted
        assert!(result.granted);
    }

    #[tokio::test]
    async fn test_check_permission_detailed_role_name() {
        let rbac = create_test_rbac_system().await;
        let user = create_test_user(UserRole::Admin);

        let result = rbac
            .check_permission_detailed(&user, "api.chat")
            .await
            .unwrap();

        assert!(result.granted);
        assert!(result.granted_by_roles.contains(&"admin".to_string()));
    }

    // ==================== check_resource_permission Tests ====================

    #[tokio::test]
    async fn test_check_resource_permission_granted() {
        let rbac = create_test_rbac_system().await;
        let user_perms = vec!["users.read".to_string(), "users.write".to_string()];

        let result = rbac.check_resource_permission(&user_perms, "users", "read");
        assert!(result);
    }

    #[tokio::test]
    async fn test_check_resource_permission_denied() {
        let rbac = create_test_rbac_system().await;
        let user_perms = vec!["users.read".to_string()];

        let result = rbac.check_resource_permission(&user_perms, "users", "delete");
        assert!(!result);
    }

    #[tokio::test]
    async fn test_check_resource_permission_api() {
        let rbac = create_test_rbac_system().await;
        let user_perms = vec!["api.chat".to_string(), "api.embeddings".to_string()];

        assert!(rbac.check_resource_permission(&user_perms, "api", "chat"));
        assert!(rbac.check_resource_permission(&user_perms, "api", "embeddings"));
        assert!(!rbac.check_resource_permission(&user_perms, "api", "images"));
    }

    #[tokio::test]
    async fn test_check_resource_permission_with_wildcard() {
        let rbac = create_test_rbac_system().await;
        let user_perms = vec!["*".to_string()];

        let result = rbac.check_resource_permission(&user_perms, "any", "action");
        assert!(result);
    }

    // ==================== is_admin Tests ====================

    #[tokio::test]
    async fn test_is_admin_super_admin() {
        let rbac = create_enabled_rbac_system().await;
        let user = create_test_user(UserRole::SuperAdmin);

        assert!(rbac.is_admin(&user));
    }

    #[tokio::test]
    async fn test_is_admin_admin() {
        let rbac = create_enabled_rbac_system().await;
        let user = create_test_user(UserRole::Admin);

        assert!(rbac.is_admin(&user));
    }

    #[tokio::test]
    async fn test_is_admin_manager() {
        let rbac = create_enabled_rbac_system().await;
        let user = create_test_user(UserRole::Manager);

        assert!(!rbac.is_admin(&user));
    }

    #[tokio::test]
    async fn test_is_admin_user() {
        let rbac = create_enabled_rbac_system().await;
        let user = create_test_user(UserRole::User);

        assert!(!rbac.is_admin(&user));
    }

    #[tokio::test]
    async fn test_is_admin_viewer() {
        let rbac = create_enabled_rbac_system().await;
        let user = create_test_user(UserRole::Viewer);

        assert!(!rbac.is_admin(&user));
    }

    // ==================== get_permission Tests ====================

    #[tokio::test]
    async fn test_get_permission_existing() {
        let rbac = create_test_rbac_system().await;

        let perm = rbac.get_permission("users.read");
        assert!(perm.is_some());
        assert_eq!(perm.unwrap().name, "users.read");
    }

    #[tokio::test]
    async fn test_get_permission_nonexistent() {
        let rbac = create_test_rbac_system().await;

        let perm = rbac.get_permission("nonexistent.permission");
        assert!(perm.is_none());
    }

    #[tokio::test]
    async fn test_get_permission_all_defaults() {
        let rbac = create_test_rbac_system().await;

        let default_perms = [
            "users.read",
            "users.write",
            "users.delete",
            "teams.read",
            "teams.write",
            "teams.delete",
            "api.chat",
            "api.embeddings",
            "api.images",
            "api_keys.read",
            "api_keys.write",
            "api_keys.delete",
            "analytics.read",
            "system.admin",
        ];

        for perm_name in &default_perms {
            let perm = rbac.get_permission(perm_name);
            assert!(perm.is_some(), "Permission {} should exist", perm_name);
        }
    }

    // ==================== add_permission Tests ====================

    #[tokio::test]
    async fn test_add_permission_success() {
        let mut rbac = create_test_rbac_system().await;

        let permission = Permission {
            name: "custom.read".to_string(),
            description: "Custom read permission".to_string(),
            resource: "custom".to_string(),
            action: "read".to_string(),
            is_system: false,
        };

        let result = rbac.add_permission(permission);
        assert!(result.is_ok());

        let added = rbac.get_permission("custom.read");
        assert!(added.is_some());
    }

    #[tokio::test]
    async fn test_add_permission_system_fails() {
        let mut rbac = create_test_rbac_system().await;

        let permission = Permission {
            name: "system.custom".to_string(),
            description: "System permission".to_string(),
            resource: "system".to_string(),
            action: "custom".to_string(),
            is_system: true,
        };

        let result = rbac.add_permission(permission);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Cannot modify system permissions")
        );
    }

    #[tokio::test]
    async fn test_add_permission_overwrites() {
        let mut rbac = create_test_rbac_system().await;

        let perm1 = Permission {
            name: "custom.perm".to_string(),
            description: "First".to_string(),
            resource: "custom".to_string(),
            action: "perm".to_string(),
            is_system: false,
        };
        rbac.add_permission(perm1).unwrap();

        let perm2 = Permission {
            name: "custom.perm".to_string(),
            description: "Second".to_string(),
            resource: "custom".to_string(),
            action: "perm".to_string(),
            is_system: false,
        };
        rbac.add_permission(perm2).unwrap();

        let result = rbac.get_permission("custom.perm").unwrap();
        assert_eq!(result.description, "Second");
    }

    // ==================== Integration Tests ====================

    #[tokio::test]
    async fn test_full_permission_flow() {
        let rbac = create_test_rbac_system().await;
        let user = create_test_user(UserRole::User);

        // Get user permissions
        let user_perms = rbac.get_user_permissions(&user).await.unwrap();

        // Check if user can chat
        let can_chat = rbac.check_permissions(&user_perms, &["api.chat".to_string()]);
        assert!(can_chat);

        // Check if user can delete users
        let can_delete = rbac.check_permissions(&user_perms, &["users.delete".to_string()]);
        assert!(!can_delete);

        // Detailed check
        let detailed = rbac
            .check_permission_detailed(&user, "api.chat")
            .await
            .unwrap();
        assert!(detailed.granted);
    }

    #[tokio::test]
    async fn test_admin_vs_user_permissions() {
        let rbac = create_test_rbac_system().await;

        let admin = create_test_user(UserRole::Admin);
        let user = create_test_user(UserRole::User);

        let admin_perms = rbac.get_user_permissions(&admin).await.unwrap();
        let user_perms = rbac.get_user_permissions(&user).await.unwrap();

        // Admin should have more permissions
        assert!(admin_perms.len() > user_perms.len());

        // Both can chat
        assert!(rbac.check_permissions(&admin_perms, &["api.chat".to_string()]));
        assert!(rbac.check_permissions(&user_perms, &["api.chat".to_string()]));

        // Only admin can write users
        assert!(rbac.check_permissions(&admin_perms, &["users.write".to_string()]));
        assert!(!rbac.check_permissions(&user_perms, &["users.write".to_string()]));
    }

    #[tokio::test]
    async fn test_custom_permission_usage() {
        let mut rbac = create_test_rbac_system().await;

        // Add custom permission
        let custom = Permission {
            name: "billing.read".to_string(),
            description: "Read billing info".to_string(),
            resource: "billing".to_string(),
            action: "read".to_string(),
            is_system: false,
        };
        rbac.add_permission(custom).unwrap();

        // Verify it exists
        assert!(rbac.get_permission("billing.read").is_some());

        // Can be used in permission checks
        let user_perms = vec!["billing.read".to_string()];
        assert!(rbac.check_permissions(&user_perms, &["billing.read".to_string()]));
    }
}
