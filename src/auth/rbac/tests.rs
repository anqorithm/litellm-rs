//! Tests for RBAC functionality

#[cfg(test)]
mod tests {
    use crate::auth::rbac::RbacSystem;
    use crate::auth::rbac::types::{Permission, PermissionCheck, Role};
    use crate::config::models::auth::RbacConfig;
    use crate::core::teams::TeamRole;
    use std::collections::HashSet;

    async fn create_test_rbac() -> RbacSystem {
        let config = RbacConfig {
            enabled: true,
            default_role: "user".to_string(),
            admin_roles: vec!["admin".to_string(), "super_admin".to_string()],
        };

        RbacSystem::new(&config).await.unwrap()
    }

    #[tokio::test]
    async fn test_rbac_initialization() {
        let rbac = create_test_rbac().await;

        assert!(!rbac.list_roles().is_empty());
        assert!(!rbac.list_permissions().is_empty());
        assert!(rbac.get_role("user").is_some());
        assert!(rbac.get_role("admin").is_some());
        assert!(rbac.get_permission("api.chat").is_some());
    }

    #[tokio::test]
    async fn test_rbac_default_roles_exist() {
        let rbac = create_test_rbac().await;

        // Check all default roles exist
        assert!(rbac.get_role("super_admin").is_some());
        assert!(rbac.get_role("admin").is_some());
        assert!(rbac.get_role("manager").is_some());
        assert!(rbac.get_role("user").is_some());
        assert!(rbac.get_role("viewer").is_some());
        assert!(rbac.get_role("api_user").is_some());
    }

    #[tokio::test]
    async fn test_rbac_default_permissions_exist() {
        let rbac = create_test_rbac().await;

        // Check all default permissions exist
        assert!(rbac.get_permission("users.read").is_some());
        assert!(rbac.get_permission("users.write").is_some());
        assert!(rbac.get_permission("users.delete").is_some());
        assert!(rbac.get_permission("teams.read").is_some());
        assert!(rbac.get_permission("teams.write").is_some());
        assert!(rbac.get_permission("api.chat").is_some());
        assert!(rbac.get_permission("api.embeddings").is_some());
        assert!(rbac.get_permission("api.images").is_some());
        assert!(rbac.get_permission("api_keys.read").is_some());
        assert!(rbac.get_permission("analytics.read").is_some());
        assert!(rbac.get_permission("system.admin").is_some());
    }

    #[tokio::test]
    async fn test_permission_checking() {
        let rbac = create_test_rbac().await;

        let user_permissions = vec!["api.chat".to_string(), "api.embeddings".to_string()];
        let required_permissions = vec!["api.chat".to_string()];

        assert!(rbac.check_permissions(&user_permissions, &required_permissions));

        let required_permissions = vec!["system.admin".to_string()];
        assert!(!rbac.check_permissions(&user_permissions, &required_permissions));
    }

    #[tokio::test]
    async fn test_permission_checking_multiple_required() {
        let rbac = create_test_rbac().await;

        let user_permissions = vec![
            "api.chat".to_string(),
            "api.embeddings".to_string(),
            "api.images".to_string(),
        ];
        let required_permissions = vec!["api.chat".to_string(), "api.embeddings".to_string()];

        assert!(rbac.check_permissions(&user_permissions, &required_permissions));

        // Missing one permission
        let required_permissions = vec!["api.chat".to_string(), "system.admin".to_string()];
        assert!(!rbac.check_permissions(&user_permissions, &required_permissions));
    }

    #[tokio::test]
    async fn test_admin_permissions() {
        let rbac = create_test_rbac().await;

        let admin_permissions = vec!["system.admin".to_string()];
        let any_permission = vec!["api.chat".to_string()];

        assert!(rbac.check_permissions(&admin_permissions, &any_permission));
    }

    #[tokio::test]
    async fn test_wildcard_permission() {
        let rbac = create_test_rbac().await;

        let wildcard_permissions = vec!["*".to_string()];
        let any_permission = vec!["api.chat".to_string(), "users.delete".to_string()];

        assert!(rbac.check_permissions(&wildcard_permissions, &any_permission));
    }

    #[tokio::test]
    async fn test_check_any_permission() {
        let rbac = create_test_rbac().await;

        let user_permissions = vec!["api.chat".to_string()];
        let required_permissions = vec!["api.chat".to_string(), "system.admin".to_string()];

        // User has at least one of the required permissions
        assert!(rbac.check_any_permission(&user_permissions, &required_permissions));

        // User has none of the required permissions
        let required_permissions = vec!["users.delete".to_string(), "system.admin".to_string()];
        assert!(!rbac.check_any_permission(&user_permissions, &required_permissions));
    }

    #[tokio::test]
    async fn test_check_resource_permission() {
        let rbac = create_test_rbac().await;

        let user_permissions = vec!["api.chat".to_string(), "users.read".to_string()];

        assert!(rbac.check_resource_permission(&user_permissions, "api", "chat"));
        assert!(rbac.check_resource_permission(&user_permissions, "users", "read"));
        assert!(!rbac.check_resource_permission(&user_permissions, "users", "delete"));
    }

    #[tokio::test]
    async fn test_add_custom_role() {
        let mut rbac = create_test_rbac().await;

        let custom_role = Role {
            name: "custom_role".to_string(),
            description: "Custom test role".to_string(),
            permissions: ["api.chat".to_string()].iter().cloned().collect(),
            parent_roles: HashSet::new(),
            is_system: false,
        };

        let result = rbac.add_role(custom_role);
        assert!(result.is_ok());
        assert!(rbac.get_role("custom_role").is_some());
    }

    #[tokio::test]
    async fn test_cannot_add_system_role() {
        let mut rbac = create_test_rbac().await;

        let system_role = Role {
            name: "fake_system".to_string(),
            description: "Fake system role".to_string(),
            permissions: HashSet::new(),
            parent_roles: HashSet::new(),
            is_system: true,
        };

        let result = rbac.add_role(system_role);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_add_custom_permission() {
        let mut rbac = create_test_rbac().await;

        let custom_permission = Permission {
            name: "custom.action".to_string(),
            description: "Custom action".to_string(),
            resource: "custom".to_string(),
            action: "action".to_string(),
            is_system: false,
        };

        let result = rbac.add_permission(custom_permission);
        assert!(result.is_ok());
        assert!(rbac.get_permission("custom.action").is_some());
    }

    #[tokio::test]
    async fn test_cannot_add_system_permission() {
        let mut rbac = create_test_rbac().await;

        let system_permission = Permission {
            name: "fake.system".to_string(),
            description: "Fake system permission".to_string(),
            resource: "fake".to_string(),
            action: "system".to_string(),
            is_system: true,
        };

        let result = rbac.add_permission(system_permission);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_team_role_to_permissions_owner() {
        let rbac = create_test_rbac().await;

        let permissions = rbac.team_role_to_permissions(&TeamRole::Owner);
        assert!(permissions.contains(&"teams.read".to_string()));
        assert!(permissions.contains(&"teams.write".to_string()));
        assert!(permissions.contains(&"teams.delete".to_string()));
        assert!(permissions.contains(&"api_keys.delete".to_string()));
    }

    #[tokio::test]
    async fn test_team_role_to_permissions_admin() {
        let rbac = create_test_rbac().await;

        let permissions = rbac.team_role_to_permissions(&TeamRole::Admin);
        assert!(permissions.contains(&"teams.read".to_string()));
        assert!(permissions.contains(&"teams.write".to_string()));
        assert!(!permissions.contains(&"teams.delete".to_string()));
    }

    #[tokio::test]
    async fn test_team_role_to_permissions_member() {
        let rbac = create_test_rbac().await;

        let permissions = rbac.team_role_to_permissions(&TeamRole::Member);
        assert!(permissions.contains(&"teams.read".to_string()));
        assert!(permissions.contains(&"api_keys.read".to_string()));
        assert!(!permissions.contains(&"api_keys.write".to_string()));
    }

    #[tokio::test]
    async fn test_team_role_to_permissions_viewer() {
        let rbac = create_test_rbac().await;

        let permissions = rbac.team_role_to_permissions(&TeamRole::Viewer);
        assert!(permissions.contains(&"teams.read".to_string()));
        assert!(permissions.contains(&"analytics.read".to_string()));
        assert!(!permissions.contains(&"api_keys.read".to_string()));
    }

    #[test]
    fn test_role_creation() {
        let role = Role {
            name: "test_role".to_string(),
            description: "Test role".to_string(),
            permissions: ["api.chat".to_string()].iter().cloned().collect(),
            parent_roles: HashSet::new(),
            is_system: false,
        };

        assert_eq!(role.name, "test_role");
        assert!(role.permissions.contains("api.chat"));
        assert!(!role.is_system);
    }

    #[test]
    fn test_role_with_parent_roles() {
        let mut parent_roles = HashSet::new();
        parent_roles.insert("user".to_string());

        let role = Role {
            name: "extended_user".to_string(),
            description: "Extended user role".to_string(),
            permissions: ["api.images".to_string()].iter().cloned().collect(),
            parent_roles,
            is_system: false,
        };

        assert!(role.parent_roles.contains("user"));
        assert_eq!(role.parent_roles.len(), 1);
    }

    #[test]
    fn test_permission_creation() {
        let permission = Permission {
            name: "test.permission".to_string(),
            description: "Test permission".to_string(),
            resource: "test".to_string(),
            action: "permission".to_string(),
            is_system: false,
        };

        assert_eq!(permission.name, "test.permission");
        assert_eq!(permission.resource, "test");
        assert_eq!(permission.action, "permission");
    }

    #[test]
    fn test_permission_check_result_granted() {
        let check = PermissionCheck {
            granted: true,
            granted_by_roles: vec!["admin".to_string()],
            denial_reason: None,
        };

        assert!(check.granted);
        assert!(check.denial_reason.is_none());
        assert_eq!(check.granted_by_roles.len(), 1);
    }

    #[test]
    fn test_permission_check_result_denied() {
        let check = PermissionCheck {
            granted: false,
            granted_by_roles: vec![],
            denial_reason: Some("Missing permission: system.admin".to_string()),
        };

        assert!(!check.granted);
        assert!(check.denial_reason.is_some());
        assert!(check.granted_by_roles.is_empty());
    }

    #[test]
    fn test_role_clone() {
        let role = Role {
            name: "test".to_string(),
            description: "Test".to_string(),
            permissions: ["read".to_string()].iter().cloned().collect(),
            parent_roles: HashSet::new(),
            is_system: false,
        };

        let cloned = role.clone();
        assert_eq!(role.name, cloned.name);
        assert_eq!(role.permissions, cloned.permissions);
    }

    #[test]
    fn test_permission_clone() {
        let permission = Permission {
            name: "test.read".to_string(),
            description: "Test read".to_string(),
            resource: "test".to_string(),
            action: "read".to_string(),
            is_system: false,
        };

        let cloned = permission.clone();
        assert_eq!(permission.name, cloned.name);
        assert_eq!(permission.resource, cloned.resource);
    }

    #[tokio::test]
    async fn test_list_roles_count() {
        let rbac = create_test_rbac().await;

        let roles = rbac.list_roles();
        // Should have at least 6 default roles
        assert!(roles.len() >= 6);
    }

    #[tokio::test]
    async fn test_list_permissions_count() {
        let rbac = create_test_rbac().await;

        let permissions = rbac.list_permissions();
        // Should have at least 13 default permissions
        assert!(permissions.len() >= 13);
    }

    #[tokio::test]
    async fn test_super_admin_has_all_permissions() {
        let rbac = create_test_rbac().await;

        let super_admin_role = rbac.get_role("super_admin").unwrap();
        let all_permissions = rbac.list_permissions();

        // Super admin should have all permissions
        for permission in all_permissions {
            assert!(
                super_admin_role.permissions.contains(&permission.name),
                "Super admin missing permission: {}",
                permission.name
            );
        }
    }

    #[tokio::test]
    async fn test_empty_permissions_check() {
        let rbac = create_test_rbac().await;

        let user_permissions: Vec<String> = vec![];
        let required_permissions = vec!["api.chat".to_string()];

        assert!(!rbac.check_permissions(&user_permissions, &required_permissions));
    }

    #[tokio::test]
    async fn test_empty_required_permissions() {
        let rbac = create_test_rbac().await;

        let user_permissions = vec!["api.chat".to_string()];
        let required_permissions: Vec<String> = vec![];

        // Empty required = always passes
        assert!(rbac.check_permissions(&user_permissions, &required_permissions));
    }
}
