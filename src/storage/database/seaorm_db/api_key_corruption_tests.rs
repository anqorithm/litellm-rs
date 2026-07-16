use super::super::entities::{self, api_key};
use super::types::SeaOrmDatabase;
use crate::config::models::storage::DatabaseConfig;
use crate::core::models::user::types::User;
use crate::core::models::{ApiKey, Metadata, RateLimits, UsageStats};
use sea_orm::prelude::Expr;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::collections::HashMap;
use uuid::Uuid;

const CORRUPT_JSON: &str = "SENSITIVE_CORRUPT_JSON";
const KEY_HASH: &str = "SENSITIVE_KEY_HASH";
const KEY_PREFIX: &str = "SENSITIVE_KEY_PREFIX";

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

async fn create_owner(db: &SeaOrmDatabase) -> Uuid {
    let user = User::new(
        "api-key-json-owner".to_string(),
        "api-key-json-owner@example.com".to_string(),
        "password-hash".to_string(),
    );
    let user_id = user.id();
    db.create_user(&user).await.expect("owner should be stored");
    user_id
}

fn valid_api_key(user_id: Uuid, team_id: Uuid) -> ApiKey {
    let mut metadata = Metadata::new();
    metadata.extra = HashMap::from([("source".to_string(), serde_json::json!("test"))]);

    ApiKey {
        metadata,
        name: "corruption-test".to_string(),
        key_hash: KEY_HASH.to_string(),
        key_prefix: KEY_PREFIX.to_string(),
        user_id: Some(user_id),
        team_id: Some(team_id),
        permissions: vec!["api.chat".to_string()],
        rate_limits: Some(RateLimits {
            rpm: Some(2),
            tpm: None,
            rpd: None,
            tpd: None,
            concurrent: None,
        }),
        expires_at: None,
        is_active: true,
        last_used_at: None,
        usage_stats: UsageStats::default(),
    }
}

async fn set_json_column(db: &SeaOrmDatabase, key_id: Uuid, column: api_key::Column, value: &str) {
    entities::ApiKey::update_many()
        .col_expr(column, Expr::value(value.to_string()))
        .filter(api_key::Column::Id.eq(key_id))
        .exec(&db.db)
        .await
        .expect("test should update JSON column");
}

fn assert_redacted_field_error(error: impl std::fmt::Display, field: &str) {
    let message = error.to_string();
    assert!(message.contains(field), "missing field context: {message}");
    for sensitive in [CORRUPT_JSON, KEY_HASH, KEY_PREFIX] {
        assert!(
            !message.contains(sensitive),
            "error leaked sensitive fixture {sensitive}: {message}"
        );
    }
}

#[tokio::test]
async fn every_malformed_json_field_fails_without_defaulting_or_leaking() {
    let db = create_database().await;
    let owner_id = create_owner(&db).await;
    let key = valid_api_key(owner_id, Uuid::new_v4());
    let key_id = key.metadata.id;
    db.create_api_key(&key).await.expect("key should be stored");

    let stored = entities::ApiKey::find_by_id(key_id)
        .one(&db.db)
        .await
        .expect("raw key lookup should succeed")
        .expect("raw key should exist");
    let cases = [
        (
            api_key::Column::Permissions,
            "permissions",
            stored.permissions,
        ),
        (
            api_key::Column::RateLimits,
            "rate_limits",
            stored.rate_limits.expect("rate limits should be stored"),
        ),
        (
            api_key::Column::UsageStats,
            "usage_stats",
            stored.usage_stats,
        ),
        (
            api_key::Column::Extra,
            "extra",
            stored.extra.expect("extra metadata should be stored"),
        ),
    ];

    for (column, field, valid_value) in cases {
        set_json_column(&db, key_id, column, CORRUPT_JSON).await;
        let error = db
            .find_api_key_by_id(key_id)
            .await
            .expect_err("corrupt JSON should fail conversion");
        assert_redacted_field_error(error, field);
        set_json_column(&db, key_id, column, &valid_value).await;
    }
}

#[tokio::test]
async fn malformed_rate_limits_fail_authoritative_lookups_and_lists() {
    let db = create_database().await;
    let owner_id = create_owner(&db).await;
    let team_id = Uuid::new_v4();
    let key = valid_api_key(owner_id, team_id);
    let key_id = key.metadata.id;
    db.create_api_key(&key).await.expect("key should be stored");
    set_json_column(&db, key_id, api_key::Column::RateLimits, CORRUPT_JSON).await;

    let lookup_results = [
        db.find_api_key_by_hash(KEY_HASH).await.map(|_| ()),
        db.find_api_key_by_id(key_id).await.map(|_| ()),
        db.list_api_keys_by_user(owner_id).await.map(|_| ()),
        db.list_api_keys_by_team(team_id).await.map(|_| ()),
        db.list_api_keys(None, None, None).await.map(|_| ()),
    ];
    for result in lookup_results {
        let error = result.expect_err("corrupt rate limits should fail every read path");
        assert_redacted_field_error(error, "rate_limits");
    }
}

#[tokio::test]
async fn failed_usage_update_preserves_corrupt_row() {
    let db = create_database().await;
    let owner_id = create_owner(&db).await;
    let key = valid_api_key(owner_id, Uuid::new_v4());
    let key_id = key.metadata.id;
    db.create_api_key(&key).await.expect("key should be stored");
    set_json_column(&db, key_id, api_key::Column::UsageStats, CORRUPT_JSON).await;

    let error = db
        .update_api_key_usage(key_id, 1, 10, 0.25, false, None)
        .await
        .expect_err("usage update should fail on corrupt persisted counters");
    assert_redacted_field_error(error, "usage_stats");

    let stored = entities::ApiKey::find_by_id(key_id)
        .one(&db.db)
        .await
        .expect("raw key lookup should succeed")
        .expect("raw key should exist");
    assert_eq!(stored.usage_stats, CORRUPT_JSON);
    assert_eq!(stored.version, key.metadata.version as i32);
}

#[tokio::test]
async fn null_optional_json_fields_and_valid_round_trip_remain_supported() {
    let db = create_database().await;
    let owner_id = create_owner(&db).await;
    let mut key = valid_api_key(owner_id, Uuid::new_v4());
    key.rate_limits = None;
    key.metadata.extra.clear();
    let key_id = key.metadata.id;
    db.create_api_key(&key).await.expect("key should be stored");

    let loaded = db
        .find_api_key_by_id(key_id)
        .await
        .expect("valid key should load")
        .expect("valid key should exist");
    assert!(loaded.rate_limits.is_none());
    assert!(loaded.metadata.extra.is_empty());
    assert_eq!(loaded.permissions, key.permissions);
    assert_eq!(loaded.usage_stats.total_requests, 0);
}
