use super::team_repository::SeaOrmTeamRepository;
use super::types::SeaOrmDatabase;
use crate::config::models::storage::DatabaseConfig;
use crate::core::models::team::{Team, TeamMember, TeamRole, TeamStatus};
use crate::core::teams::repository::TeamRepository;
use crate::core::user_management::{
    Team as LegacyTeam, TeamMember as LegacyTeamMember, TeamRole as LegacyTeamRole,
    TeamSettings as LegacyTeamSettings, User as LegacyUser,
    UserPreferences as LegacyUserPreferences, UserRole as LegacyUserRole,
};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

async fn create_repository_with_db() -> (SeaOrmTeamRepository, Arc<SeaOrmDatabase>) {
    let db = Arc::new(
        SeaOrmDatabase::new(&DatabaseConfig {
            enabled: false,
            ..DatabaseConfig::default()
        })
        .await
        .expect("failed to create in-memory database"),
    );
    db.migrate().await.expect("failed to run migrations");
    (SeaOrmTeamRepository::new(Arc::clone(&db)), db)
}

async fn create_repository() -> SeaOrmTeamRepository {
    let (repo, _) = create_repository_with_db().await;
    repo
}

fn legacy_team(name: &str, owner_id: Uuid) -> LegacyTeam {
    LegacyTeam {
        team_id: Uuid::new_v4().to_string(),
        team_name: name.to_string(),
        description: Some(format!("legacy {}", name)),
        organization_id: Some(Uuid::new_v4().to_string()),
        members: vec![LegacyTeamMember {
            user_id: owner_id.to_string(),
            role: LegacyTeamRole::Owner,
            joined_at: Utc::now(),
            is_active: true,
        }],
        permissions: vec!["api.chat".to_string()],
        models: vec!["gpt-4".to_string()],
        max_budget: Some(1000.0),
        spend: 42.0,
        budget_duration: Some("1m".to_string()),
        budget_reset_at: Some(Utc::now()),
        metadata: HashMap::from([("source".to_string(), "legacy-test".to_string())]),
        is_active: true,
        created_at: Utc::now(),
        settings: LegacyTeamSettings::default(),
    }
}

fn legacy_user(user_id: Uuid, email: &str) -> LegacyUser {
    LegacyUser {
        user_id: user_id.to_string(),
        email: email.to_string(),
        display_name: Some(email.to_string()),
        first_name: None,
        last_name: None,
        role: LegacyUserRole::User,
        teams: vec![],
        permissions: vec![],
        metadata: HashMap::new(),
        max_budget: None,
        spend: 0.0,
        budget_duration: None,
        budget_reset_at: None,
        is_active: true,
        created_at: Utc::now(),
        last_login_at: None,
        preferences: LegacyUserPreferences::default(),
    }
}

#[tokio::test]
async fn test_list_and_count_exclude_deleted_teams() {
    let repo = create_repository().await;

    let active_a = Team::new("active-a".to_string(), None);
    let active_b = Team::new("active-b".to_string(), None);
    let mut deleted = Team::new("deleted-a".to_string(), None);
    deleted.status = TeamStatus::Deleted;

    repo.create(active_a).await.unwrap();
    repo.create(deleted).await.unwrap();
    repo.create(active_b).await.unwrap();

    let (teams, total) = repo.list(0, 10).await.unwrap();
    assert_eq!(total, 2);
    assert_eq!(teams.len(), 2);

    let names: Vec<String> = teams.into_iter().map(|t| t.name).collect();
    assert!(names.contains(&"active-a".to_string()));
    assert!(names.contains(&"active-b".to_string()));
    assert!(!names.contains(&"deleted-a".to_string()));

    let count = repo.count().await.unwrap();
    assert_eq!(count, 2);
}

#[tokio::test]
async fn test_pagination_applies_after_deleted_filtering() {
    let repo = create_repository().await;

    let mut team_b_deleted = Team::new("team-b".to_string(), None);
    team_b_deleted.status = TeamStatus::Deleted;
    let mut team_e_deleted = Team::new("team-e".to_string(), None);
    team_e_deleted.status = TeamStatus::Deleted;

    repo.create(Team::new("team-a".to_string(), None))
        .await
        .unwrap();
    repo.create(team_b_deleted).await.unwrap();
    repo.create(Team::new("team-c".to_string(), None))
        .await
        .unwrap();
    repo.create(Team::new("team-d".to_string(), None))
        .await
        .unwrap();
    repo.create(team_e_deleted).await.unwrap();
    repo.create(Team::new("team-f".to_string(), None))
        .await
        .unwrap();

    let (teams, total) = repo.list(1, 2).await.unwrap();
    assert_eq!(total, 4);
    assert_eq!(teams.len(), 2);
    assert_eq!(teams[0].name, "team-c");
    assert_eq!(teams[1].name, "team-d");
}

#[tokio::test]
async fn test_legacy_user_management_team_is_visible_through_canonical_repository() {
    let (repo, db) = create_repository_with_db().await;
    let owner_id = Uuid::new_v4();
    let legacy = legacy_team("legacy-visible", owner_id);
    let legacy_id = Uuid::parse_str(&legacy.team_id).unwrap();

    db.create_team(&legacy).await.unwrap();

    let fetched = repo.get(legacy_id).await.unwrap().unwrap();
    assert_eq!(fetched.id(), legacy_id);
    assert_eq!(fetched.name, "legacy-visible");
    assert_eq!(fetched.description, legacy.description);
    assert!(fetched.metadata.extra.contains_key("legacy_um_team"));
    assert_eq!(fetched.usage_stats.total_cost, 42.0);

    let by_name = repo.get_by_name("legacy-visible").await.unwrap().unwrap();
    assert_eq!(by_name.id(), legacy_id);

    let (teams, total) = repo.list(0, 10).await.unwrap();
    assert_eq!(total, 1);
    assert_eq!(teams.len(), 1);
    assert_eq!(teams[0].id(), legacy_id);
}

#[tokio::test]
async fn test_legacy_user_management_team_members_are_copied_to_canonical_members() {
    let (repo, db) = create_repository_with_db().await;
    let owner_id = Uuid::new_v4();
    let legacy = legacy_team("legacy-members", owner_id);
    let legacy_id = Uuid::parse_str(&legacy.team_id).unwrap();

    db.create_team(&legacy).await.unwrap();

    let user_teams = repo.get_user_teams(owner_id).await.unwrap();
    assert_eq!(user_teams.len(), 1);
    assert_eq!(user_teams[0].id(), legacy_id);

    let member = repo.get_member(legacy_id, owner_id).await.unwrap().unwrap();
    assert_eq!(member.team_id, legacy_id);
    assert_eq!(member.user_id, owner_id);
}

#[tokio::test]
async fn test_legacy_member_removal_removes_backfilled_canonical_member() {
    let (repo, db) = create_repository_with_db().await;
    let owner_id = Uuid::new_v4();
    let removed_user_id = Uuid::new_v4();
    let mut legacy = legacy_team("legacy-member-removal", owner_id);
    let legacy_id = Uuid::parse_str(&legacy.team_id).unwrap();
    legacy.members.push(LegacyTeamMember {
        user_id: removed_user_id.to_string(),
        role: LegacyTeamRole::Member,
        joined_at: Utc::now(),
        is_active: true,
    });

    db.create_team(&legacy).await.unwrap();
    assert_eq!(repo.get_user_teams(removed_user_id).await.unwrap().len(), 1);

    legacy
        .members
        .retain(|member| member.user_id != removed_user_id.to_string());
    db.update_team(&legacy).await.unwrap();

    assert!(
        repo.get_user_teams(removed_user_id)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        repo.get_member(legacy_id, removed_user_id)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn test_inactive_legacy_members_are_not_backfilled_to_canonical_members() {
    let (repo, db) = create_repository_with_db().await;
    let owner_id = Uuid::new_v4();
    let inactive_user_id = Uuid::new_v4();
    let mut legacy = legacy_team("legacy-inactive-member", owner_id);
    let legacy_id = Uuid::parse_str(&legacy.team_id).unwrap();
    legacy.members.push(LegacyTeamMember {
        user_id: inactive_user_id.to_string(),
        role: LegacyTeamRole::Admin,
        joined_at: Utc::now(),
        is_active: false,
    });

    db.create_team(&legacy).await.unwrap();

    assert!(
        repo.get_member(legacy_id, inactive_user_id)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        repo.get_user_teams(inactive_user_id)
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn test_delete_does_not_resurrect_backfilled_legacy_team() {
    let (repo, db) = create_repository_with_db().await;
    let owner_id = Uuid::new_v4();
    let legacy = legacy_team("legacy-delete", owner_id);
    let legacy_id = Uuid::parse_str(&legacy.team_id).unwrap();

    db.create_team(&legacy).await.unwrap();
    assert!(repo.get(legacy_id).await.unwrap().is_some());

    repo.delete(legacy_id).await.unwrap();

    assert!(repo.get(legacy_id).await.unwrap().is_none());
    let (teams, total) = repo.list(0, 10).await.unwrap();
    assert_eq!(total, 0);
    assert!(teams.is_empty());
}

#[tokio::test]
async fn test_legacy_name_conflict_does_not_return_wrong_team_for_id_lookup() {
    let (repo, db) = create_repository_with_db().await;
    let canonical = Team::new("shared-name".to_string(), None);
    let canonical_id = canonical.id();
    repo.create(canonical).await.unwrap();

    let legacy = legacy_team("shared-name", Uuid::new_v4());
    let legacy_id = Uuid::parse_str(&legacy.team_id).unwrap();
    db.create_team(&legacy).await.unwrap();

    assert!(repo.get(legacy_id).await.unwrap().is_none());
    let by_name = repo.get_by_name("shared-name").await.unwrap().unwrap();
    assert_eq!(by_name.id(), canonical_id);
}

#[tokio::test]
async fn test_canonical_team_create_is_visible_through_legacy_user_management_tables() {
    let (repo, db) = create_repository_with_db().await;
    let mut canonical = Team::new(
        "canonical-visible".to_string(),
        Some("Canonical".to_string()),
    );
    canonical.description = Some("created through canonical repository".to_string());
    let canonical_id = canonical.id();

    repo.create(canonical).await.unwrap();

    let legacy = db
        .get_team(&canonical_id.to_string())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(legacy.team_id, canonical_id.to_string());
    assert_eq!(legacy.team_name, "canonical-visible");
    assert_eq!(
        legacy.description,
        Some("created through canonical repository".to_string())
    );
}

#[tokio::test]
async fn test_canonical_update_preserves_legacy_settings_and_organization_id() {
    let (repo, db) = create_repository_with_db().await;
    let owner_id = Uuid::new_v4();
    let mut legacy = legacy_team("legacy-update-preserve", owner_id);
    let legacy_id = Uuid::parse_str(&legacy.team_id).unwrap();
    let organization_id = legacy.organization_id.clone();
    legacy.settings.default_model = Some("claude-3-5-sonnet".to_string());
    legacy.settings.auto_approve_members = false;
    legacy.settings.require_approval_for_high_cost = true;
    legacy.settings.high_cost_threshold = Some(25.0);
    db.create_team(&legacy).await.unwrap();

    let mut canonical = repo.get(legacy_id).await.unwrap().unwrap();
    canonical.description = Some("updated through canonical repository".to_string());

    repo.update(canonical).await.unwrap();

    let updated = db.get_team(&legacy_id.to_string()).await.unwrap().unwrap();
    assert_eq!(updated.organization_id, organization_id);
    assert_eq!(
        updated.description,
        Some("updated through canonical repository".to_string())
    );
    assert_eq!(
        updated.settings.default_model,
        Some("claude-3-5-sonnet".to_string())
    );
    assert!(!updated.settings.auto_approve_members);
    assert!(updated.settings.require_approval_for_high_cost);
    assert_eq!(updated.settings.high_cost_threshold, Some(25.0));
}

#[tokio::test]
async fn test_canonical_update_preserves_existing_legacy_organization_id() {
    let (repo, db) = create_repository_with_db().await;
    let mut canonical = Team::new("canonical-existing-org".to_string(), None);
    let team_id = canonical.id();
    repo.create(canonical.clone()).await.unwrap();

    let mut legacy = db.get_team(&team_id.to_string()).await.unwrap().unwrap();
    legacy.organization_id = Some("legacy-org-kept".to_string());
    db.update_team(&legacy).await.unwrap();

    canonical.description = Some("canonical update without legacy org metadata".to_string());
    repo.update(canonical).await.unwrap();

    let updated = db.get_team(&team_id.to_string()).await.unwrap().unwrap();
    assert_eq!(updated.organization_id, Some("legacy-org-kept".to_string()));
    assert_eq!(
        updated.description,
        Some("canonical update without legacy org metadata".to_string())
    );
}

#[tokio::test]
async fn test_canonical_member_changes_update_legacy_team_and_user_membership() {
    let (repo, db) = create_repository_with_db().await;
    let user_id = Uuid::new_v4();
    db.um_create_user(&legacy_user(user_id, "member@example.com"))
        .await
        .unwrap();

    let canonical = repo
        .create(Team::new("canonical-members".to_string(), None))
        .await
        .unwrap();
    let team_id = canonical.id();
    let member = TeamMember::new(team_id, user_id, TeamRole::Member, None);

    repo.add_member(member).await.unwrap();

    let legacy = db.get_team(&team_id.to_string()).await.unwrap().unwrap();
    assert_eq!(legacy.members.len(), 1);
    assert_eq!(legacy.members[0].user_id, user_id.to_string());
    let legacy_user = db.get_user(&user_id.to_string()).await.unwrap().unwrap();
    assert_eq!(legacy_user.teams, vec![team_id.to_string()]);

    repo.remove_member(team_id, user_id).await.unwrap();

    let legacy = db.get_team(&team_id.to_string()).await.unwrap().unwrap();
    assert!(legacy.members.is_empty());
    let legacy_user = db.get_user(&user_id.to_string()).await.unwrap().unwrap();
    assert!(legacy_user.teams.is_empty());
}

#[tokio::test]
async fn test_delete_legacy_only_team_removes_legacy_user_membership_without_readthrough() {
    let (repo, db) = create_repository_with_db().await;
    let owner_id = Uuid::new_v4();
    let legacy = legacy_team("legacy-delete-direct", owner_id);
    let legacy_id = Uuid::parse_str(&legacy.team_id).unwrap();
    let mut user = legacy_user(owner_id, "legacy-delete-direct@example.com");
    user.teams.push(legacy_id.to_string());

    db.um_create_user(&user).await.unwrap();
    db.create_team(&legacy).await.unwrap();

    repo.delete(legacy_id).await.unwrap();

    assert!(db.get_team(&legacy_id.to_string()).await.unwrap().is_none());
    let legacy_user = db.get_user(&owner_id.to_string()).await.unwrap().unwrap();
    assert!(legacy_user.teams.is_empty());
}

#[tokio::test]
async fn test_canonical_team_delete_removes_legacy_user_memberships() {
    let (repo, db) = create_repository_with_db().await;
    let user_id = Uuid::new_v4();
    db.um_create_user(&legacy_user(user_id, "delete-member@example.com"))
        .await
        .unwrap();

    let team = repo
        .create(Team::new("canonical-delete-membership".to_string(), None))
        .await
        .unwrap();
    let team_id = team.id();
    repo.add_member(TeamMember::new(team_id, user_id, TeamRole::Member, None))
        .await
        .unwrap();

    repo.delete(team_id).await.unwrap();

    assert!(db.get_team(&team_id.to_string()).await.unwrap().is_none());
    let legacy_user = db.get_user(&user_id.to_string()).await.unwrap().unwrap();
    assert!(legacy_user.teams.is_empty());
}
