//! User module tests

#[cfg(test)]
use crate::core::models::user::session::{SessionType, UserSession};
use crate::core::models::user::types::{User, UserRole};
use uuid::Uuid;

#[test]
fn test_user_creation() {
    let user = User::new(
        "testuser".to_string(),
        "test@example.com".to_string(),
        "hashed_password".to_string(),
    );

    assert_eq!(user.username, "testuser");
    assert_eq!(user.email, "test@example.com");
    assert!(matches!(user.role, UserRole::User));
    assert!(!user.is_active());
}

#[test]
fn test_user_role_hierarchy() {
    let mut user = User::new(
        "admin".to_string(),
        "admin@example.com".to_string(),
        "hashed_password".to_string(),
    );
    user.role = UserRole::Admin;

    assert!(user.has_role(&UserRole::Admin));
    assert!(user.has_role(&UserRole::User));
    assert!(user.has_role(&UserRole::Viewer));
    assert!(!user.has_role(&UserRole::SuperAdmin));
}

#[test]
fn test_team_management() {
    let mut user = User::new(
        "testuser".to_string(),
        "test@example.com".to_string(),
        "hashed_password".to_string(),
    );

    let team_id = Uuid::new_v4();
    assert!(!user.is_in_team(team_id));

    user.add_to_team(team_id);
    assert!(user.is_in_team(team_id));

    user.remove_from_team(team_id);
    assert!(!user.is_in_team(team_id));
}

#[test]
fn test_session_expiry() {
    let user_id = Uuid::new_v4();
    let token = "test_token".to_string();
    let expires_at = chrono::Utc::now() - chrono::Duration::hours(1); // Expired

    let session = UserSession::new(user_id, token, SessionType::Web, expires_at);
    assert!(session.is_expired());
}
