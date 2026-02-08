//! Helper methods for RBAC operations

use std::collections::HashSet;

use super::system::RbacSystem;
use super::types::Role;

pub(super) trait RbacHelpers {
    /// Get all permissions for a role (including inherited)
    fn get_role_permissions(&self, role: &Role) -> HashSet<String>;
}

impl RbacHelpers for RbacSystem {
    /// Get all permissions for a role (including inherited)
    fn get_role_permissions(&self, role: &Role) -> HashSet<String> {
        let mut permissions = role.permissions.clone();

        // Add permissions from parent roles
        for parent_role_name in &role.parent_roles {
            if let Some(parent_role) = self.roles.get(parent_role_name) {
                permissions.extend(self.get_role_permissions(parent_role));
            }
        }

        permissions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::models::auth::RbacConfig;

    // ==================== Helper Functions ====================

    async fn create_test_rbac_system() -> RbacSystem {
        let config = RbacConfig::default();
        RbacSystem::new(&config).await.unwrap()
    }

    fn create_role_with_permissions(name: &str, permissions: Vec<&str>) -> Role {
        Role {
            name: name.to_string(),
            description: format!("Test role: {}", name),
            permissions: permissions.into_iter().map(|s| s.to_string()).collect(),
            parent_roles: HashSet::new(),
            is_system: false,
        }
    }

    fn create_role_with_parent(name: &str, permissions: Vec<&str>, parents: Vec<&str>) -> Role {
        Role {
            name: name.to_string(),
            description: format!("Test role: {}", name),
            permissions: permissions.into_iter().map(|s| s.to_string()).collect(),
            parent_roles: parents.into_iter().map(|s| s.to_string()).collect(),
            is_system: false,
        }
    }

    // ==================== get_role_permissions Tests ====================

    #[tokio::test]
    async fn test_get_role_permissions_simple() {
        let rbac = create_test_rbac_system().await;
        let role = create_role_with_permissions("test", vec!["read", "write"]);

        let perms = rbac.get_role_permissions(&role);

        assert_eq!(perms.len(), 2);
        assert!(perms.contains("read"));
        assert!(perms.contains("write"));
    }

    #[tokio::test]
    async fn test_get_role_permissions_empty() {
        let rbac = create_test_rbac_system().await;
        let role = create_role_with_permissions("empty", vec![]);

        let perms = rbac.get_role_permissions(&role);

        assert!(perms.is_empty());
    }

    #[tokio::test]
    async fn test_get_role_permissions_single() {
        let rbac = create_test_rbac_system().await;
        let role = create_role_with_permissions("single", vec!["api.chat"]);

        let perms = rbac.get_role_permissions(&role);

        assert_eq!(perms.len(), 1);
        assert!(perms.contains("api.chat"));
    }

    #[tokio::test]
    async fn test_get_role_permissions_many() {
        let rbac = create_test_rbac_system().await;
        let role = create_role_with_permissions(
            "many",
            vec![
                "api.chat",
                "api.embeddings",
                "api.images",
                "users.read",
                "teams.read",
            ],
        );

        let perms = rbac.get_role_permissions(&role);

        assert_eq!(perms.len(), 5);
        assert!(perms.contains("api.chat"));
        assert!(perms.contains("teams.read"));
    }

    #[tokio::test]
    async fn test_get_role_permissions_with_existing_parent() {
        let rbac = create_test_rbac_system().await;

        // Create role with 'user' as parent (which exists in default rbac)
        let role = create_role_with_parent("child", vec!["custom.perm"], vec!["user"]);

        let perms = rbac.get_role_permissions(&role);

        // Should have own permission plus inherited from 'user'
        assert!(perms.contains("custom.perm"));
        assert!(perms.contains("api.chat")); // user role has api.chat
    }

    #[tokio::test]
    async fn test_get_role_permissions_with_nonexistent_parent() {
        let rbac = create_test_rbac_system().await;

        // Create role with nonexistent parent
        let role = create_role_with_parent("child", vec!["own.perm"], vec!["nonexistent"]);

        let perms = rbac.get_role_permissions(&role);

        // Should only have own permissions (parent not found)
        assert_eq!(perms.len(), 1);
        assert!(perms.contains("own.perm"));
    }

    #[tokio::test]
    async fn test_get_role_permissions_super_admin() {
        let rbac = create_test_rbac_system().await;
        let super_admin = rbac.get_role("super_admin").unwrap();

        let perms = rbac.get_role_permissions(super_admin);

        // Super admin should have all permissions
        assert!(perms.contains("users.read"));
        assert!(perms.contains("users.write"));
        assert!(perms.contains("users.delete"));
        assert!(perms.contains("teams.read"));
        assert!(perms.contains("api.chat"));
        assert!(perms.contains("system.admin"));
    }

    #[tokio::test]
    async fn test_get_role_permissions_user_role() {
        let rbac = create_test_rbac_system().await;
        let user = rbac.get_role("user").unwrap();

        let perms = rbac.get_role_permissions(user);

        assert!(perms.contains("api.chat"));
        assert!(perms.contains("api.embeddings"));
        assert!(perms.contains("api_keys.read"));
        assert_eq!(perms.len(), 3);
    }

    #[tokio::test]
    async fn test_get_role_permissions_viewer_role() {
        let rbac = create_test_rbac_system().await;
        let viewer = rbac.get_role("viewer").unwrap();

        let perms = rbac.get_role_permissions(viewer);

        assert!(perms.contains("users.read"));
        assert!(perms.contains("teams.read"));
        assert!(perms.contains("api_keys.read"));
        assert!(perms.contains("analytics.read"));
        assert!(!perms.contains("api.chat")); // Viewer shouldn't have API access
    }

    #[tokio::test]
    async fn test_get_role_permissions_api_user() {
        let rbac = create_test_rbac_system().await;
        let api_user = rbac.get_role("api_user").unwrap();

        let perms = rbac.get_role_permissions(api_user);

        assert!(perms.contains("api.chat"));
        assert!(perms.contains("api.embeddings"));
        assert!(perms.contains("api.images"));
        assert_eq!(perms.len(), 3);
    }

    // ==================== Inheritance Tests ====================

    #[tokio::test]
    async fn test_permission_inheritance_single_level() {
        let mut rbac = create_test_rbac_system().await;

        // Add child role that inherits from 'user'
        let child = create_role_with_parent("child_user", vec!["extra.perm"], vec!["user"]);
        rbac.add_role(child.clone()).unwrap();

        let perms = rbac.get_role_permissions(&child);

        // Should have own + inherited
        assert!(perms.contains("extra.perm"));
        assert!(perms.contains("api.chat"));
        assert!(perms.contains("api.embeddings"));
    }

    #[tokio::test]
    async fn test_permission_inheritance_multiple_parents() {
        let mut rbac = create_test_rbac_system().await;

        // Create role with multiple parents
        let multi_parent =
            create_role_with_parent("multi_parent", vec!["own.perm"], vec!["user", "viewer"]);
        rbac.add_role(multi_parent.clone()).unwrap();

        let perms = rbac.get_role_permissions(&multi_parent);

        // Should have own + inherited from both parents
        assert!(perms.contains("own.perm"));
        assert!(perms.contains("api.chat")); // from user
        assert!(perms.contains("users.read")); // from viewer
    }

    #[tokio::test]
    async fn test_permission_deduplication() {
        let mut rbac = create_test_rbac_system().await;

        // Create role with duplicate permission
        let role = create_role_with_parent(
            "dup_test",
            vec!["api.chat"], // This is also in 'user'
            vec!["user"],
        );
        rbac.add_role(role.clone()).unwrap();

        let perms = rbac.get_role_permissions(&role);

        // Count of api.chat should be 1 (deduped)
        let count = perms.iter().filter(|p| *p == "api.chat").count();
        assert_eq!(count, 1);
    }

    // ==================== Edge Cases ====================

    #[tokio::test]
    async fn test_self_referencing_parent_ignored() {
        let rbac = create_test_rbac_system().await;

        // Create role that references itself (shouldn't cause infinite loop)
        let self_ref = Role {
            name: "self_ref".to_string(),
            description: "Self-referencing role".to_string(),
            permissions: ["perm1".to_string()].into_iter().collect(),
            parent_roles: ["self_ref".to_string()].into_iter().collect(),
            is_system: false,
        };

        // This should not cause stack overflow - parent not found in rbac.roles
        let perms = rbac.get_role_permissions(&self_ref);
        assert!(perms.contains("perm1"));
    }

    #[tokio::test]
    async fn test_permissions_are_hashset() {
        let rbac = create_test_rbac_system().await;
        let role = create_role_with_permissions("test", vec!["a", "b", "a", "c", "b"]);

        let perms = rbac.get_role_permissions(&role);

        // HashSet should dedupe
        assert_eq!(perms.len(), 3);
    }

    // ==================== Integration Tests ====================

    #[tokio::test]
    async fn test_all_default_roles_have_permissions() {
        let rbac = create_test_rbac_system().await;

        for role in rbac.list_roles() {
            let perms = rbac.get_role_permissions(role);
            assert!(
                !perms.is_empty() || role.name == "viewer",
                "Role {} should have permissions",
                role.name
            );
        }
    }

    #[tokio::test]
    async fn test_super_admin_has_most_permissions() {
        let rbac = create_test_rbac_system().await;

        let super_admin = rbac.get_role("super_admin").unwrap();
        let super_admin_perms = rbac.get_role_permissions(super_admin);

        for role in rbac.list_roles() {
            if role.name != "super_admin" {
                let role_perms = rbac.get_role_permissions(role);
                assert!(
                    super_admin_perms.len() >= role_perms.len(),
                    "Super admin should have at least as many permissions as {}",
                    role.name
                );
            }
        }
    }

    #[tokio::test]
    async fn test_role_permissions_immutable() {
        let rbac = create_test_rbac_system().await;
        let role = rbac.get_role("user").unwrap();

        let perms1 = rbac.get_role_permissions(role);
        let perms2 = rbac.get_role_permissions(role);

        assert_eq!(perms1, perms2);
    }

    #[tokio::test]
    async fn test_custom_role_inheritance_chain() {
        let mut rbac = create_test_rbac_system().await;

        // Create a chain: custom3 -> custom2 -> custom1 -> user
        let custom1 = create_role_with_parent("custom1", vec!["custom1.perm"], vec!["user"]);
        rbac.add_role(custom1).unwrap();

        let custom2 = create_role_with_parent("custom2", vec!["custom2.perm"], vec!["custom1"]);
        rbac.add_role(custom2).unwrap();

        let custom3 = create_role_with_parent("custom3", vec!["custom3.perm"], vec!["custom2"]);
        rbac.add_role(custom3.clone()).unwrap();

        let perms = rbac.get_role_permissions(&custom3);

        // Should have all permissions through the chain
        assert!(perms.contains("custom3.perm"));
        assert!(perms.contains("custom2.perm"));
        assert!(perms.contains("custom1.perm"));
        assert!(perms.contains("api.chat")); // from user
    }
}
