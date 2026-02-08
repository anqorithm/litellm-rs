//! Role management methods

use crate::core::models::team::TeamRole;
use crate::core::models::user::types::UserRole;
use crate::utils::error::error::{GatewayError, Result};

use super::system::RbacSystem;
use super::types::Role;

impl RbacSystem {
    /// Get role by name
    pub fn get_role(&self, role_name: &str) -> Option<&Role> {
        self.roles.get(role_name)
    }

    /// Add custom role
    pub fn add_role(&mut self, role: Role) -> Result<()> {
        if role.is_system {
            return Err(GatewayError::auth("Cannot modify system roles"));
        }

        self.roles.insert(role.name.clone(), role);
        Ok(())
    }

    /// Convert UserRole to string
    pub(super) fn user_role_to_string(&self, role: &UserRole) -> String {
        match role {
            UserRole::SuperAdmin => "super_admin".to_string(),
            UserRole::Admin => "admin".to_string(),
            UserRole::Manager => "manager".to_string(),
            UserRole::User => "user".to_string(),
            UserRole::Viewer => "viewer".to_string(),
            UserRole::ApiUser => "api_user".to_string(),
        }
    }

    /// Convert TeamRole to permissions
    pub fn team_role_to_permissions(&self, role: &TeamRole) -> Vec<String> {
        match role {
            TeamRole::Owner => vec![
                "teams.read".to_string(),
                "teams.write".to_string(),
                "teams.delete".to_string(),
                "users.read".to_string(),
                "users.write".to_string(),
                "api_keys.read".to_string(),
                "api_keys.write".to_string(),
                "api_keys.delete".to_string(),
                "analytics.read".to_string(),
            ],
            TeamRole::Admin => vec![
                "teams.read".to_string(),
                "teams.write".to_string(),
                "users.read".to_string(),
                "users.write".to_string(),
                "api_keys.read".to_string(),
                "api_keys.write".to_string(),
                "analytics.read".to_string(),
            ],
            TeamRole::Manager => vec![
                "teams.read".to_string(),
                "users.read".to_string(),
                "api_keys.read".to_string(),
                "api_keys.write".to_string(),
                "analytics.read".to_string(),
            ],
            TeamRole::Member => vec![
                "teams.read".to_string(),
                "api_keys.read".to_string(),
                "analytics.read".to_string(),
            ],
            TeamRole::Viewer => vec!["teams.read".to_string(), "analytics.read".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RbacConfig;
    use crate::core::models::user::types::UserRole;
    use std::collections::HashSet;

    // ==================== Helper Functions ====================

    async fn create_test_rbac_system() -> RbacSystem {
        let config = RbacConfig::default();
        RbacSystem::new(&config).await.unwrap()
    }

    fn create_test_role(name: &str, is_system: bool) -> Role {
        Role {
            name: name.to_string(),
            description: format!("Test role: {}", name),
            permissions: HashSet::new(),
            parent_roles: HashSet::new(),
            is_system,
        }
    }

    // ==================== get_role Tests ====================

    #[tokio::test]
    async fn test_get_role_existing() {
        let rbac = create_test_rbac_system().await;

        let role = rbac.get_role("super_admin");
        assert!(role.is_some());
        assert_eq!(role.unwrap().name, "super_admin");
    }

    #[tokio::test]
    async fn test_get_role_admin() {
        let rbac = create_test_rbac_system().await;

        let role = rbac.get_role("admin");
        assert!(role.is_some());
        assert_eq!(role.unwrap().name, "admin");
        assert!(role.unwrap().is_system);
    }

    #[tokio::test]
    async fn test_get_role_user() {
        let rbac = create_test_rbac_system().await;

        let role = rbac.get_role("user");
        assert!(role.is_some());
        let user_role = role.unwrap();
        assert!(user_role.permissions.contains("api.chat"));
    }

    #[tokio::test]
    async fn test_get_role_nonexistent() {
        let rbac = create_test_rbac_system().await;

        let role = rbac.get_role("nonexistent_role");
        assert!(role.is_none());
    }

    #[tokio::test]
    async fn test_get_role_empty_name() {
        let rbac = create_test_rbac_system().await;

        let role = rbac.get_role("");
        assert!(role.is_none());
    }

    #[tokio::test]
    async fn test_get_all_default_roles() {
        let rbac = create_test_rbac_system().await;

        let roles = [
            "super_admin",
            "admin",
            "manager",
            "user",
            "viewer",
            "api_user",
        ];
        for role_name in &roles {
            let role = rbac.get_role(role_name);
            assert!(role.is_some(), "Role {} should exist", role_name);
        }
    }

    // ==================== add_role Tests ====================

    #[tokio::test]
    async fn test_add_role_success() {
        let mut rbac = create_test_rbac_system().await;
        let custom_role = create_test_role("custom_role", false);

        let result = rbac.add_role(custom_role);
        assert!(result.is_ok());

        let added = rbac.get_role("custom_role");
        assert!(added.is_some());
    }

    #[tokio::test]
    async fn test_add_role_with_permissions() {
        let mut rbac = create_test_rbac_system().await;

        let mut permissions = HashSet::new();
        permissions.insert("api.chat".to_string());
        permissions.insert("api.embeddings".to_string());

        let role = Role {
            name: "api_access".to_string(),
            description: "Custom API access role".to_string(),
            permissions,
            parent_roles: HashSet::new(),
            is_system: false,
        };

        let result = rbac.add_role(role);
        assert!(result.is_ok());

        let added = rbac.get_role("api_access").unwrap();
        assert!(added.permissions.contains("api.chat"));
        assert!(added.permissions.contains("api.embeddings"));
    }

    #[tokio::test]
    async fn test_add_role_system_role_fails() {
        let mut rbac = create_test_rbac_system().await;
        let system_role = create_test_role("new_system", true);

        let result = rbac.add_role(system_role);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Cannot modify system roles"));
    }

    #[tokio::test]
    async fn test_add_role_overwrites_existing() {
        let mut rbac = create_test_rbac_system().await;

        // Add a role
        let role1 = Role {
            name: "test_role".to_string(),
            description: "First version".to_string(),
            permissions: HashSet::new(),
            parent_roles: HashSet::new(),
            is_system: false,
        };
        rbac.add_role(role1).unwrap();

        // Add same role with different description
        let role2 = Role {
            name: "test_role".to_string(),
            description: "Second version".to_string(),
            permissions: HashSet::new(),
            parent_roles: HashSet::new(),
            is_system: false,
        };
        rbac.add_role(role2).unwrap();

        let result = rbac.get_role("test_role").unwrap();
        assert_eq!(result.description, "Second version");
    }

    #[tokio::test]
    async fn test_add_role_with_parent_roles() {
        let mut rbac = create_test_rbac_system().await;

        let mut parent_roles = HashSet::new();
        parent_roles.insert("user".to_string());

        let role = Role {
            name: "enhanced_user".to_string(),
            description: "User with extra permissions".to_string(),
            permissions: HashSet::new(),
            parent_roles,
            is_system: false,
        };

        let result = rbac.add_role(role);
        assert!(result.is_ok());

        let added = rbac.get_role("enhanced_user").unwrap();
        assert!(added.parent_roles.contains("user"));
    }

    // ==================== user_role_to_string Tests ====================

    #[tokio::test]
    async fn test_user_role_to_string_super_admin() {
        let rbac = create_test_rbac_system().await;
        let result = rbac.user_role_to_string(&UserRole::SuperAdmin);
        assert_eq!(result, "super_admin");
    }

    #[tokio::test]
    async fn test_user_role_to_string_admin() {
        let rbac = create_test_rbac_system().await;
        let result = rbac.user_role_to_string(&UserRole::Admin);
        assert_eq!(result, "admin");
    }

    #[tokio::test]
    async fn test_user_role_to_string_manager() {
        let rbac = create_test_rbac_system().await;
        let result = rbac.user_role_to_string(&UserRole::Manager);
        assert_eq!(result, "manager");
    }

    #[tokio::test]
    async fn test_user_role_to_string_user() {
        let rbac = create_test_rbac_system().await;
        let result = rbac.user_role_to_string(&UserRole::User);
        assert_eq!(result, "user");
    }

    #[tokio::test]
    async fn test_user_role_to_string_viewer() {
        let rbac = create_test_rbac_system().await;
        let result = rbac.user_role_to_string(&UserRole::Viewer);
        assert_eq!(result, "viewer");
    }

    #[tokio::test]
    async fn test_user_role_to_string_api_user() {
        let rbac = create_test_rbac_system().await;
        let result = rbac.user_role_to_string(&UserRole::ApiUser);
        assert_eq!(result, "api_user");
    }

    #[tokio::test]
    async fn test_user_role_to_string_all_roles() {
        let rbac = create_test_rbac_system().await;

        let roles = vec![
            (UserRole::SuperAdmin, "super_admin"),
            (UserRole::Admin, "admin"),
            (UserRole::Manager, "manager"),
            (UserRole::User, "user"),
            (UserRole::Viewer, "viewer"),
            (UserRole::ApiUser, "api_user"),
        ];

        for (role, expected) in roles {
            let result = rbac.user_role_to_string(&role);
            assert_eq!(result, expected, "Failed for {:?}", role);
        }
    }

    // ==================== team_role_to_permissions Tests ====================

    #[tokio::test]
    async fn test_team_role_owner_permissions() {
        let rbac = create_test_rbac_system().await;
        let perms = rbac.team_role_to_permissions(&TeamRole::Owner);

        assert!(perms.contains(&"teams.read".to_string()));
        assert!(perms.contains(&"teams.write".to_string()));
        assert!(perms.contains(&"teams.delete".to_string()));
        assert!(perms.contains(&"users.read".to_string()));
        assert!(perms.contains(&"users.write".to_string()));
        assert!(perms.contains(&"api_keys.read".to_string()));
        assert!(perms.contains(&"api_keys.write".to_string()));
        assert!(perms.contains(&"api_keys.delete".to_string()));
        assert!(perms.contains(&"analytics.read".to_string()));
        assert_eq!(perms.len(), 9);
    }

    #[tokio::test]
    async fn test_team_role_admin_permissions() {
        let rbac = create_test_rbac_system().await;
        let perms = rbac.team_role_to_permissions(&TeamRole::Admin);

        assert!(perms.contains(&"teams.read".to_string()));
        assert!(perms.contains(&"teams.write".to_string()));
        assert!(perms.contains(&"users.read".to_string()));
        assert!(perms.contains(&"users.write".to_string()));
        assert!(perms.contains(&"api_keys.read".to_string()));
        assert!(perms.contains(&"api_keys.write".to_string()));
        assert!(perms.contains(&"analytics.read".to_string()));
        // Admin should NOT have delete permissions
        assert!(!perms.contains(&"teams.delete".to_string()));
        assert!(!perms.contains(&"api_keys.delete".to_string()));
        assert_eq!(perms.len(), 7);
    }

    #[tokio::test]
    async fn test_team_role_manager_permissions() {
        let rbac = create_test_rbac_system().await;
        let perms = rbac.team_role_to_permissions(&TeamRole::Manager);

        assert!(perms.contains(&"teams.read".to_string()));
        assert!(perms.contains(&"users.read".to_string()));
        assert!(perms.contains(&"api_keys.read".to_string()));
        assert!(perms.contains(&"api_keys.write".to_string()));
        assert!(perms.contains(&"analytics.read".to_string()));
        // Manager should NOT have teams.write or users.write
        assert!(!perms.contains(&"teams.write".to_string()));
        assert!(!perms.contains(&"users.write".to_string()));
        assert_eq!(perms.len(), 5);
    }

    #[tokio::test]
    async fn test_team_role_member_permissions() {
        let rbac = create_test_rbac_system().await;
        let perms = rbac.team_role_to_permissions(&TeamRole::Member);

        assert!(perms.contains(&"teams.read".to_string()));
        assert!(perms.contains(&"api_keys.read".to_string()));
        assert!(perms.contains(&"analytics.read".to_string()));
        // Member should NOT have write permissions
        assert!(!perms.contains(&"teams.write".to_string()));
        assert!(!perms.contains(&"api_keys.write".to_string()));
        assert_eq!(perms.len(), 3);
    }

    #[tokio::test]
    async fn test_team_role_viewer_permissions() {
        let rbac = create_test_rbac_system().await;
        let perms = rbac.team_role_to_permissions(&TeamRole::Viewer);

        assert!(perms.contains(&"teams.read".to_string()));
        assert!(perms.contains(&"analytics.read".to_string()));
        // Viewer should only have read permissions
        assert!(!perms.contains(&"teams.write".to_string()));
        assert!(!perms.contains(&"api_keys.read".to_string()));
        assert_eq!(perms.len(), 2);
    }

    #[tokio::test]
    async fn test_team_role_hierarchy_permission_count() {
        let rbac = create_test_rbac_system().await;

        let owner_perms = rbac.team_role_to_permissions(&TeamRole::Owner);
        let admin_perms = rbac.team_role_to_permissions(&TeamRole::Admin);
        let manager_perms = rbac.team_role_to_permissions(&TeamRole::Manager);
        let member_perms = rbac.team_role_to_permissions(&TeamRole::Member);
        let viewer_perms = rbac.team_role_to_permissions(&TeamRole::Viewer);

        // Owner has most permissions
        assert!(owner_perms.len() > admin_perms.len());
        assert!(admin_perms.len() > manager_perms.len());
        assert!(manager_perms.len() > member_perms.len());
        assert!(member_perms.len() > viewer_perms.len());
    }

    #[tokio::test]
    async fn test_all_team_roles_have_teams_read() {
        let rbac = create_test_rbac_system().await;

        let roles = vec![
            TeamRole::Owner,
            TeamRole::Admin,
            TeamRole::Manager,
            TeamRole::Member,
            TeamRole::Viewer,
        ];

        for role in roles {
            let perms = rbac.team_role_to_permissions(&role);
            assert!(
                perms.contains(&"teams.read".to_string()),
                "{:?} should have teams.read permission",
                role
            );
        }
    }

    #[tokio::test]
    async fn test_only_owner_has_delete_permissions() {
        let rbac = create_test_rbac_system().await;

        let owner_perms = rbac.team_role_to_permissions(&TeamRole::Owner);
        assert!(owner_perms.contains(&"teams.delete".to_string()));
        assert!(owner_perms.contains(&"api_keys.delete".to_string()));

        let non_owner_roles = vec![
            TeamRole::Admin,
            TeamRole::Manager,
            TeamRole::Member,
            TeamRole::Viewer,
        ];

        for role in non_owner_roles {
            let perms = rbac.team_role_to_permissions(&role);
            assert!(
                !perms.contains(&"teams.delete".to_string()),
                "{:?} should not have teams.delete",
                role
            );
            assert!(
                !perms.contains(&"api_keys.delete".to_string()),
                "{:?} should not have api_keys.delete",
                role
            );
        }
    }

    // ==================== Integration Tests ====================

    #[tokio::test]
    async fn test_user_role_maps_to_rbac_role() {
        let rbac = create_test_rbac_system().await;

        let user_role = UserRole::Admin;
        let role_name = rbac.user_role_to_string(&user_role);
        let rbac_role = rbac.get_role(&role_name);

        assert!(rbac_role.is_some());
        assert_eq!(rbac_role.unwrap().name, "admin");
    }

    #[tokio::test]
    async fn test_all_user_roles_map_to_rbac_roles() {
        let rbac = create_test_rbac_system().await;

        let user_roles = vec![
            UserRole::SuperAdmin,
            UserRole::Admin,
            UserRole::Manager,
            UserRole::User,
            UserRole::Viewer,
            UserRole::ApiUser,
        ];

        for user_role in user_roles {
            let role_name = rbac.user_role_to_string(&user_role);
            let rbac_role = rbac.get_role(&role_name);
            assert!(
                rbac_role.is_some(),
                "UserRole::{:?} should map to an existing RBAC role",
                user_role
            );
        }
    }

    #[tokio::test]
    async fn test_custom_role_workflow() {
        let mut rbac = create_test_rbac_system().await;

        // Create custom role
        let mut permissions = HashSet::new();
        permissions.insert("api.chat".to_string());
        permissions.insert("api.embeddings".to_string());

        let custom_role = Role {
            name: "api_only".to_string(),
            description: "API access only".to_string(),
            permissions,
            parent_roles: HashSet::new(),
            is_system: false,
        };

        // Add the role
        rbac.add_role(custom_role).unwrap();

        // Verify it exists
        let role = rbac.get_role("api_only").unwrap();
        assert!(!role.is_system);
        assert!(role.permissions.contains("api.chat"));
        assert!(role.permissions.contains("api.embeddings"));
        assert_eq!(role.permissions.len(), 2);
    }

    #[tokio::test]
    async fn test_permission_comparison_user_vs_team() {
        let rbac = create_test_rbac_system().await;

        // Get user role permissions
        let user_role = rbac.get_role("user").unwrap();

        // Get team member permissions
        let team_member_perms = rbac.team_role_to_permissions(&TeamRole::Member);

        // User has api.chat, team member has teams.read
        assert!(user_role.permissions.contains("api.chat"));
        assert!(team_member_perms.contains(&"teams.read".to_string()));
    }
}
