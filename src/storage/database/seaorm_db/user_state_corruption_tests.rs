use super::super::entities;
use super::types::SeaOrmDatabase;
use super::user_repository_tests::{canonical_user, create_database};
use crate::core::models::user::types::{User, UserRole, UserStatus};
use sea_orm::{ActiveModelTrait, Set};
use uuid::Uuid;

const INVALID_ROLE: &str = "sentinel-invalid-role-do-not-expose";
const INVALID_STATUS: &str = "sentinel-invalid-status-do-not-expose";

async fn replace_role(db: &SeaOrmDatabase, user_id: Uuid, role: &str) {
    entities::user::ActiveModel {
        id: Set(user_id),
        role: Set(role.to_string()),
        ..Default::default()
    }
    .update(&db.db)
    .await
    .expect("test should replace persisted role");
}

async fn replace_status(db: &SeaOrmDatabase, user_id: Uuid, status: &str) {
    entities::user::ActiveModel {
        id: Set(user_id),
        status: Set(status.to_string()),
        ..Default::default()
    }
    .update(&db.db)
    .await
    .expect("test should replace persisted status");
}

async fn assert_all_lookup_paths_fail(db: &SeaOrmDatabase, user: &User, field: &str) {
    for result in [
        db.find_user_by_id(user.id()).await,
        db.find_user_by_username(&user.username).await,
        db.find_user_by_email(&user.email).await,
    ] {
        let error = result.expect_err("corrupt canonical user must fail lookup");
        let rendered = error.to_string();
        assert!(rendered.contains(field));
        assert!(!rendered.contains(INVALID_ROLE));
        assert!(!rendered.contains(INVALID_STATUS));
        assert!(!rendered.contains(&user.username));
        assert!(!rendered.contains(&user.email));
        assert!(!rendered.contains(&user.password_hash));
    }
}

#[tokio::test]
async fn malformed_role_fails_every_canonical_lookup_without_exposing_row_data() {
    let db = create_database().await;
    let user = canonical_user("corrupt-role-user", "corrupt-role@example.com");
    db.create_user(&user).await.unwrap();
    replace_role(&db, user.id(), INVALID_ROLE).await;

    assert_all_lookup_paths_fail(&db, &user, "role").await;
}

#[tokio::test]
async fn malformed_status_fails_every_canonical_lookup_without_exposing_row_data() {
    let db = create_database().await;
    let user = canonical_user("corrupt-status-user", "corrupt-status@example.com");
    db.create_user(&user).await.unwrap();
    replace_status(&db, user.id(), INVALID_STATUS).await;

    assert_all_lookup_paths_fail(&db, &user, "status").await;
}

#[tokio::test]
async fn valid_roles_and_statuses_round_trip_exactly() {
    let db = create_database().await;
    let roles = [
        UserRole::SuperAdmin,
        UserRole::Admin,
        UserRole::Manager,
        UserRole::User,
        UserRole::Viewer,
        UserRole::ApiUser,
    ];

    for (index, role) in roles.into_iter().enumerate() {
        let mut user = canonical_user(
            &format!("role-{index}"),
            &format!("role-{index}@example.com"),
        );
        user.role = role.clone();
        db.create_user(&user).await.unwrap();
        let loaded = db.find_user_by_id(user.id()).await.unwrap().unwrap();
        assert_eq!(loaded.role, role);
    }

    let statuses = [
        UserStatus::Active,
        UserStatus::Inactive,
        UserStatus::Pending,
        UserStatus::Suspended,
        UserStatus::Deleted,
    ];

    for (index, status) in statuses.into_iter().enumerate() {
        let mut user = canonical_user(
            &format!("status-{index}"),
            &format!("status-{index}@example.com"),
        );
        user.status = status.clone();
        db.create_user(&user).await.unwrap();
        let loaded = db.find_user_by_id(user.id()).await.unwrap().unwrap();
        assert!(
            std::mem::discriminant(&loaded.status) == std::mem::discriminant(&status),
            "status variant changed during round trip"
        );
    }
}

#[tokio::test]
async fn missing_canonical_users_remain_none() {
    let db = create_database().await;

    assert!(db.find_user_by_id(Uuid::new_v4()).await.unwrap().is_none());
    assert!(
        db.find_user_by_username("missing-user")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        db.find_user_by_email("missing@example.com")
            .await
            .unwrap()
            .is_none()
    );
}
