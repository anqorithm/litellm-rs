use super::super::entities;
use super::types::SeaOrmDatabase;
use crate::config::models::storage::DatabaseConfig;
use crate::core::models::user::types::{User, UserProfile, UserRateLimits, UserRole, UserStatus};
use crate::core::user_management::{
    User as LegacyUser, UserPreferences as LegacyUserPreferences, UserRole as LegacyUserRole,
};
use crate::utils::auth::crypto::password::verify_password;
use chrono::Utc;
use sea_orm::EntityTrait;
use std::collections::HashMap;
use uuid::Uuid;

async fn create_database() -> SeaOrmDatabase {
    let db = SeaOrmDatabase::new(&DatabaseConfig {
        enabled: false,
        ..DatabaseConfig::default()
    })
    .await
    .expect("failed to create in-memory database");
    db.migrate().await.expect("failed to run migrations");
    db
}

fn canonical_user(username: &str, email: &str) -> User {
    let mut user = User::new(username.to_string(), email.to_string(), "hash".to_string());
    user.status = UserStatus::Active;
    user.display_name = Some("Canonical User".to_string());
    user.profile = UserProfile {
        first_name: Some("Canonical".to_string()),
        last_name: Some("User".to_string()),
        ..UserProfile::default()
    };
    user.role = UserRole::Manager;
    user.team_ids = vec![Uuid::new_v4()];
    user.usage_stats.total_cost = 17.5;
    user.rate_limits = Some(UserRateLimits {
        rpm: None,
        tpm: None,
        rpd: None,
        tpd: None,
        concurrent: None,
        monthly_budget: Some(250.0),
    });
    user
}

fn legacy_user(user_id: Uuid, email: &str) -> LegacyUser {
    LegacyUser {
        user_id: user_id.to_string(),
        email: email.to_string(),
        display_name: Some("Legacy User".to_string()),
        first_name: Some("Legacy".to_string()),
        last_name: Some("User".to_string()),
        role: LegacyUserRole::TeamAdmin,
        teams: vec![Uuid::new_v4().to_string()],
        permissions: vec!["keys.create".to_string()],
        metadata: HashMap::new(),
        max_budget: Some(100.0),
        spend: 9.25,
        budget_duration: Some("1m".to_string()),
        budget_reset_at: Some(Utc::now()),
        is_active: true,
        created_at: Utc::now(),
        last_login_at: Some(Utc::now()),
        preferences: LegacyUserPreferences::default(),
    }
}

#[tokio::test]
async fn test_canonical_user_create_is_visible_through_legacy_user_management_tables() {
    let db = create_database().await;
    let user = canonical_user("canonical-user", "canonical@example.com");
    let user_id = user.id();

    db.create_user(&user).await.unwrap();

    let legacy = db.get_user(&user_id.to_string()).await.unwrap().unwrap();
    assert_eq!(legacy.user_id, user_id.to_string());
    assert_eq!(legacy.email, "canonical@example.com");
    assert_eq!(legacy.display_name, Some("Canonical User".to_string()));
    assert_eq!(legacy.role, LegacyUserRole::TeamAdmin);
    assert_eq!(
        legacy.teams,
        user.team_ids
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    assert_eq!(legacy.max_budget, Some(250.0));
    assert_eq!(legacy.spend, 17.5);
    assert_eq!(
        legacy.metadata.get("canonical_username"),
        Some(&"canonical-user".to_string())
    );
}

#[tokio::test]
async fn test_legacy_user_create_is_visible_through_canonical_user_repository() {
    let db = create_database().await;
    let user_id = Uuid::new_v4();
    let legacy = legacy_user(user_id, "legacy@example.com");

    db.um_create_user(&legacy).await.unwrap();

    let canonical = db.find_user_by_id(user_id).await.unwrap().unwrap();
    assert_eq!(canonical.id(), user_id);
    assert_eq!(canonical.username, "legacy@example.com");
    assert_eq!(canonical.email, "legacy@example.com");
    assert_eq!(canonical.display_name, Some("Legacy User".to_string()));
    assert_eq!(canonical.role, UserRole::Manager);
    assert!(canonical.is_active());
    assert!(canonical.last_login_at.is_some());
}

#[tokio::test]
async fn test_legacy_user_lookup_materializes_existing_canonical_user() {
    let db = create_database().await;
    let user = canonical_user("canonical-materialize", "materialize@example.com");
    let user_id = user.id();

    db.create_user(&user).await.unwrap();
    db.delete_user(&user_id.to_string()).await.unwrap();
    assert!(
        db.get_legacy_user_by_id(&user_id.to_string())
            .await
            .unwrap()
            .is_none()
    );

    let legacy = db
        .get_user_by_email("materialize@example.com")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(legacy.user_id, user_id.to_string());
    assert_eq!(legacy.email, "materialize@example.com");
}

#[tokio::test]
async fn test_canonical_user_sync_skips_legacy_email_conflict() {
    let db = create_database().await;
    let canonical = canonical_user("canonical-conflict", "conflict@example.com");
    let canonical_id = canonical.id();
    db.create_user(&canonical).await.unwrap();
    db.delete_user(&canonical_id.to_string()).await.unwrap();

    let legacy_id = Uuid::new_v4();
    db.um_create_user(&legacy_user(legacy_id, "conflict@example.com"))
        .await
        .unwrap();

    assert!(db.find_user_by_id(canonical_id).await.unwrap().is_some());
    assert!(
        db.get_user(&canonical_id.to_string())
            .await
            .unwrap()
            .is_none()
    );
    let legacy = db
        .get_legacy_user_by_email("conflict@example.com")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(legacy.user_id, legacy_id.to_string());
}

#[tokio::test]
async fn test_legacy_user_sync_skips_existing_canonical_id_email_conflict() {
    let db = create_database().await;
    let canonical = canonical_user("canonical-id-conflict", "canonical-id@example.com");
    let canonical_id = canonical.id();
    db.create_user(&canonical).await.unwrap();
    db.delete_user(&canonical_id.to_string()).await.unwrap();

    let legacy = legacy_user(canonical_id, "legacy-id@example.com");
    db.um_create_user(&legacy).await.unwrap();

    assert!(
        db.find_user_by_email("legacy-id@example.com")
            .await
            .unwrap()
            .is_none()
    );
    let canonical = db.find_user_by_id(canonical_id).await.unwrap().unwrap();
    assert_eq!(canonical.email, "canonical-id@example.com");
}

#[tokio::test]
async fn test_legacy_user_materialization_preserves_parseable_password_hash() {
    let db = create_database().await;
    let user_id = Uuid::new_v4();
    let legacy = legacy_user(user_id, "legacy-password@example.com");

    db.um_create_user(&legacy).await.unwrap();

    let canonical = db.find_user_by_id(user_id).await.unwrap().unwrap();
    assert!(canonical.password_hash.starts_with("$argon2"));
    assert!(
        !verify_password("legacy-password@example.com", &canonical.password_hash).unwrap(),
        "legacy users must not become loginable with a predictable placeholder password"
    );
}

#[tokio::test]
async fn test_legacy_user_lookup_by_canonical_username_metadata() {
    let db = create_database().await;
    let user_id = Uuid::new_v4();
    let mut legacy = legacy_user(user_id, "legacy-handle@example.com");
    legacy.metadata.insert(
        "canonical_username".to_string(),
        "legacy-handle".to_string(),
    );

    db.um_create_user(&legacy).await.unwrap();
    entities::User::delete_by_id(user_id)
        .exec(&db.db)
        .await
        .unwrap();

    let canonical = db
        .find_user_by_username("legacy-handle")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(canonical.id(), user_id);
    assert_eq!(canonical.username, "legacy-handle");
    assert_eq!(canonical.email, "legacy-handle@example.com");
}
