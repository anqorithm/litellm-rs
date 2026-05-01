//! SeaORM-backed TeamRepository implementation
//!
//! Stores `core::models::team::{Team, TeamMember}` as JSON snapshots in the
//! `teams` and `team_members` tables created by migration
//! `m20240301_000002_create_teams_table`.  Works with both SQLite and
//! PostgreSQL backends via the live `SeaOrmDatabase` connection.

use crate::core::models::team::{MemberStatus, Team, TeamMember, TeamRole, TeamStatus};
use crate::core::models::{Metadata, UsageStats};
use crate::core::teams::repository::TeamRepository;
use crate::core::user_management::{
    Team as LegacyTeam, TeamMember as LegacyTeamMember, TeamRole as LegacyTeamRole,
};
use crate::utils::error::gateway_error::{GatewayError, Result};
use async_trait::async_trait;
use sea_orm::{ConnectionTrait, DbBackend, Statement, TransactionTrait, Value};
use std::collections::HashSet;
use std::sync::Arc;
use tracing::warn;
use uuid::Uuid;

use super::types::{DatabaseBackendType, SeaOrmDatabase};

/// SeaORM-backed team repository (supports SQLite and PostgreSQL).
pub struct SeaOrmTeamRepository {
    db: Arc<SeaOrmDatabase>,
}

impl SeaOrmTeamRepository {
    /// Create a new repository wrapping the given database connection.
    pub fn new(db: Arc<SeaOrmDatabase>) -> Self {
        Self { db }
    }

    fn backend(&self) -> DbBackend {
        match self.db.backend_type {
            DatabaseBackendType::PostgreSQL => DbBackend::Postgres,
            DatabaseBackendType::SQLite => DbBackend::Sqlite,
        }
    }

    /// Return the positional placeholder for parameter `n` (1-based).
    fn ph(&self, n: usize) -> String {
        match self.db.backend_type {
            DatabaseBackendType::PostgreSQL => format!("${}", n),
            DatabaseBackendType::SQLite => "?".to_string(),
        }
    }

    fn to_json<T: serde::Serialize>(v: &T) -> Result<String> {
        serde_json::to_string(v).map_err(|e| GatewayError::Internal(e.to_string()))
    }

    fn from_json<T: serde::de::DeserializeOwned>(s: &str) -> Result<T> {
        serde_json::from_str(s).map_err(|e| GatewayError::Internal(e.to_string()))
    }

    fn invalid_legacy_uuid(kind: &str, value: &str) -> GatewayError {
        GatewayError::Internal(format!(
            "legacy user_management {} '{}' is not a valid UUID",
            kind, value
        ))
    }

    fn legacy_role_to_core(role: &LegacyTeamRole) -> TeamRole {
        match role {
            LegacyTeamRole::Owner => TeamRole::Owner,
            LegacyTeamRole::Admin => TeamRole::Admin,
            LegacyTeamRole::Member => TeamRole::Member,
            LegacyTeamRole::ReadOnly => TeamRole::Viewer,
        }
    }

    fn legacy_member_to_core(
        team_id: Uuid,
        member: &LegacyTeamMember,
    ) -> Result<Option<TeamMember>> {
        if !member.is_active {
            return Ok(None);
        }

        let user_id = match Uuid::parse_str(&member.user_id) {
            Ok(id) => id,
            Err(_) => {
                warn!(
                    legacy_user_id = %member.user_id,
                    "Skipping legacy team member because user_id is not a UUID"
                );
                return Ok(None);
            }
        };

        let mut core_member = TeamMember::new(
            team_id,
            user_id,
            Self::legacy_role_to_core(&member.role),
            None,
        );
        core_member.joined_at = member.joined_at;
        core_member.status = if member.is_active {
            MemberStatus::Active
        } else {
            MemberStatus::Left
        };
        Ok(Some(core_member))
    }

    fn legacy_team_to_core(legacy: &LegacyTeam) -> Result<(Team, Vec<TeamMember>)> {
        let team_id = Uuid::parse_str(&legacy.team_id)
            .map_err(|_| Self::invalid_legacy_uuid("team_id", &legacy.team_id))?;

        let mut metadata = Metadata::new();
        metadata.id = team_id;
        metadata.created_at = legacy.created_at;
        metadata.updated_at = legacy.created_at;
        metadata
            .extra
            .insert("legacy_um_team".to_string(), serde_json::Value::Bool(true));
        if let Some(organization_id) = &legacy.organization_id {
            metadata.extra.insert(
                "legacy_organization_id".to_string(),
                serde_json::Value::String(organization_id.clone()),
            );
        }

        let mut team = Team::new(legacy.team_name.clone(), Some(legacy.team_name.clone()));
        team.metadata = metadata;
        team.description = legacy.description.clone();
        team.status = if legacy.is_active {
            TeamStatus::Active
        } else {
            TeamStatus::Inactive
        };
        team.usage_stats = UsageStats {
            total_cost: legacy.spend,
            cost_today: legacy.spend,
            last_reset: legacy.created_at,
            ..UsageStats::default()
        };
        team.team_metadata = legacy
            .metadata
            .iter()
            .map(|(key, value)| (key.clone(), serde_json::Value::String(value.clone())))
            .collect();
        team.team_metadata.insert(
            "legacy_permissions".to_string(),
            serde_json::json!(legacy.permissions),
        );
        team.team_metadata.insert(
            "legacy_models".to_string(),
            serde_json::json!(legacy.models),
        );
        if let Some(max_budget) = legacy.max_budget {
            team.team_metadata.insert(
                "legacy_max_budget".to_string(),
                serde_json::json!(max_budget),
            );
        }
        if let Some(duration) = &legacy.budget_duration {
            team.team_metadata.insert(
                "legacy_budget_duration".to_string(),
                serde_json::Value::String(duration.clone()),
            );
        }
        if let Some(reset_at) = legacy.budget_reset_at {
            team.team_metadata.insert(
                "legacy_budget_reset_at".to_string(),
                serde_json::Value::String(reset_at.to_rfc3339()),
            );
        }

        let members = legacy
            .members
            .iter()
            .filter_map(
                |member| match Self::legacy_member_to_core(team_id, member) {
                    Ok(member) => member,
                    Err(err) => {
                        warn!(error = %err, "Skipping invalid legacy team member");
                        None
                    }
                },
            )
            .collect();

        Ok((team, members))
    }

    /// SQL predicate for filtering logically deleted teams from JSON payload.
    fn non_deleted_team_predicate(&self) -> &'static str {
        match self.db.backend_type {
            DatabaseBackendType::PostgreSQL => {
                "((data::jsonb ->> 'status') IS NULL OR (data::jsonb ->> 'status') <> 'deleted')"
            }
            DatabaseBackendType::SQLite => {
                "(json_extract(data, '$.status') IS NULL OR json_extract(data, '$.status') <> 'deleted')"
            }
        }
    }

    async fn insert_canonical_team(&self, team: &Team) -> Result<()> {
        let id = team.id().to_string();
        let name = team.name.clone();
        let data = Self::to_json(team)?;
        let sql = format!(
            "INSERT INTO teams (id, name, data) VALUES ({}, {}, {})",
            self.ph(1),
            self.ph(2),
            self.ph(3)
        );
        let stmt = Statement::from_sql_and_values(
            self.backend(),
            &sql,
            [
                Value::String(Some(Box::new(id))),
                Value::String(Some(Box::new(name))),
                Value::String(Some(Box::new(data))),
            ],
        );
        self.db.db.execute(stmt).await.map_err(GatewayError::from)?;
        Ok(())
    }

    async fn insert_canonical_member(&self, member: &TeamMember) -> Result<()> {
        let team_id = member.team_id.to_string();
        let user_id = member.user_id.to_string();
        let data = Self::to_json(member)?;
        let sql = format!(
            "INSERT INTO team_members (team_id, user_id, data) VALUES ({}, {}, {})",
            self.ph(1),
            self.ph(2),
            self.ph(3)
        );
        let stmt = Statement::from_sql_and_values(
            self.backend(),
            &sql,
            [
                Value::String(Some(Box::new(team_id))),
                Value::String(Some(Box::new(user_id))),
                Value::String(Some(Box::new(data))),
            ],
        );
        self.db.db.execute(stmt).await.map_err(GatewayError::from)?;
        Ok(())
    }

    async fn get_canonical(&self, id: Uuid) -> Result<Option<Team>> {
        let sql = format!("SELECT data FROM teams WHERE id = {}", self.ph(1));
        let stmt = Statement::from_sql_and_values(
            self.backend(),
            &sql,
            [Value::String(Some(Box::new(id.to_string())))],
        );
        match self
            .db
            .db
            .query_one(stmt)
            .await
            .map_err(GatewayError::from)?
        {
            None => Ok(None),
            Some(row) => {
                let data: String = row.try_get("", "data").map_err(GatewayError::from)?;
                Ok(Some(Self::from_json(&data)?))
            }
        }
    }

    async fn get_canonical_by_name(&self, name: &str) -> Result<Option<Team>> {
        let sql = format!("SELECT data FROM teams WHERE name = {}", self.ph(1));
        let stmt = Statement::from_sql_and_values(
            self.backend(),
            &sql,
            [Value::String(Some(Box::new(name.to_owned())))],
        );
        match self
            .db
            .db
            .query_one(stmt)
            .await
            .map_err(GatewayError::from)?
        {
            None => Ok(None),
            Some(row) => {
                let data: String = row.try_get("", "data").map_err(GatewayError::from)?;
                Ok(Some(Self::from_json(&data)?))
            }
        }
    }

    async fn get_canonical_member(
        &self,
        team_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<TeamMember>> {
        let sql = format!(
            "SELECT data FROM team_members WHERE team_id = {} AND user_id = {}",
            self.ph(1),
            self.ph(2)
        );
        let stmt = Statement::from_sql_and_values(
            self.backend(),
            &sql,
            [
                Value::String(Some(Box::new(team_id.to_string()))),
                Value::String(Some(Box::new(user_id.to_string()))),
            ],
        );
        match self
            .db
            .db
            .query_one(stmt)
            .await
            .map_err(GatewayError::from)?
        {
            None => Ok(None),
            Some(row) => {
                let data: String = row.try_get("", "data").map_err(GatewayError::from)?;
                Ok(Some(Self::from_json(&data)?))
            }
        }
    }

    async fn list_canonical_members(&self, team_id: Uuid) -> Result<Vec<TeamMember>> {
        let sql = format!(
            "SELECT data FROM team_members WHERE team_id = {}",
            self.ph(1)
        );
        let stmt = Statement::from_sql_and_values(
            self.backend(),
            &sql,
            [Value::String(Some(Box::new(team_id.to_string())))],
        );
        let rows = self
            .db
            .db
            .query_all(stmt)
            .await
            .map_err(GatewayError::from)?;
        rows.into_iter()
            .map(|row| {
                let data: String = row.try_get("", "data").map_err(GatewayError::from)?;
                Self::from_json(&data)
            })
            .collect()
    }

    async fn delete_canonical_member(&self, team_id: Uuid, user_id: Uuid) -> Result<()> {
        let sql = format!(
            "DELETE FROM team_members WHERE team_id = {} AND user_id = {}",
            self.ph(1),
            self.ph(2)
        );
        let stmt = Statement::from_sql_and_values(
            self.backend(),
            &sql,
            [
                Value::String(Some(Box::new(team_id.to_string()))),
                Value::String(Some(Box::new(user_id.to_string()))),
            ],
        );
        self.db.db.execute(stmt).await.map_err(GatewayError::from)?;
        Ok(())
    }

    async fn get_legacy_um_team(&self, id: Uuid) -> Result<Option<LegacyTeam>> {
        let sql = format!("SELECT data FROM um_teams WHERE team_id = {}", self.ph(1));
        let stmt = Statement::from_sql_and_values(
            self.backend(),
            &sql,
            [Value::String(Some(Box::new(id.to_string())))],
        );
        match self
            .db
            .db
            .query_one(stmt)
            .await
            .map_err(GatewayError::from)?
        {
            None => Ok(None),
            Some(row) => {
                let data: String = row.try_get("", "data").map_err(GatewayError::from)?;
                Ok(Some(Self::from_json(&data)?))
            }
        }
    }

    async fn list_legacy_um_teams(&self) -> Result<Vec<LegacyTeam>> {
        let sql = "SELECT data FROM um_teams ORDER BY created_at ASC";
        let stmt = Statement::from_string(self.backend(), sql);
        let rows = self
            .db
            .db
            .query_all(stmt)
            .await
            .map_err(GatewayError::from)?;

        let mut teams = Vec::with_capacity(rows.len());
        for row in rows {
            let data: String = row.try_get("", "data").map_err(GatewayError::from)?;
            match Self::from_json::<LegacyTeam>(&data) {
                Ok(team) => teams.push(team),
                Err(err) => warn!(error = %err, "Skipping invalid legacy um_teams row"),
            }
        }
        Ok(teams)
    }

    async fn ensure_legacy_team_inserted(&self, team: &Team) -> Result<bool> {
        if self.get_canonical(team.id()).await?.is_some() {
            return Ok(true);
        }

        if let Some(existing) = self.get_canonical_by_name(&team.name).await? {
            warn!(
                legacy_team_id = %team.id(),
                existing_team_id = %existing.id(),
                team_name = %team.name,
                "Skipping legacy team sync because canonical team name already exists"
            );
            return Ok(false);
        }

        match self.insert_canonical_team(team).await {
            Ok(()) => Ok(true),
            Err(err) => {
                if self.get_canonical(team.id()).await?.is_some() {
                    return Ok(true);
                }

                if let Some(existing) = self.get_canonical_by_name(&team.name).await? {
                    warn!(
                        legacy_team_id = %team.id(),
                        existing_team_id = %existing.id(),
                        team_name = %team.name,
                        "Skipping legacy team sync after concurrent canonical name insert"
                    );
                    return Ok(false);
                }

                Err(err)
            }
        }
    }

    async fn ensure_legacy_member_inserted(&self, member: &TeamMember) -> Result<()> {
        if self
            .get_canonical_member(member.team_id, member.user_id)
            .await?
            .is_some()
        {
            return Ok(());
        }

        match self.insert_canonical_member(member).await {
            Ok(()) => Ok(()),
            Err(err) => {
                if self
                    .get_canonical_member(member.team_id, member.user_id)
                    .await?
                    .is_some()
                {
                    Ok(())
                } else {
                    Err(err)
                }
            }
        }
    }

    async fn sync_legacy_members(&self, team_id: Uuid, members: Vec<TeamMember>) -> Result<()> {
        let legacy_user_ids: HashSet<Uuid> = members.iter().map(|member| member.user_id).collect();

        for member in &members {
            self.ensure_legacy_member_inserted(member).await?;
        }

        for existing in self.list_canonical_members(team_id).await? {
            if !legacy_user_ids.contains(&existing.user_id) {
                self.delete_canonical_member(team_id, existing.user_id)
                    .await?;
            }
        }

        Ok(())
    }

    async fn persist_legacy_team(&self, legacy: &LegacyTeam) -> Result<Option<Team>> {
        let (team, members) = match Self::legacy_team_to_core(legacy) {
            Ok(converted) => converted,
            Err(err) => {
                warn!(error = %err, legacy_team_id = %legacy.team_id, "Skipping legacy team sync");
                return Ok(None);
            }
        };

        if !self.ensure_legacy_team_inserted(&team).await? {
            return Ok(None);
        }

        self.sync_legacy_members(team.id(), members).await?;
        Ok(Some(team))
    }

    async fn sync_legacy_um_teams(&self) -> Result<()> {
        for legacy in self.list_legacy_um_teams().await? {
            let _ = self.persist_legacy_team(&legacy).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl TeamRepository for SeaOrmTeamRepository {
    async fn create(&self, team: Team) -> Result<Team> {
        self.insert_canonical_team(&team).await?;
        Ok(team)
    }

    async fn get(&self, id: Uuid) -> Result<Option<Team>> {
        if let Some(team) = self.get_canonical(id).await? {
            return Ok(Some(team));
        }

        match self.get_legacy_um_team(id).await? {
            Some(legacy) => self.persist_legacy_team(&legacy).await,
            None => Ok(None),
        }
    }

    async fn get_by_name(&self, name: &str) -> Result<Option<Team>> {
        if let Some(team) = self.get_canonical_by_name(name).await? {
            return Ok(Some(team));
        }

        for legacy in self.list_legacy_um_teams().await? {
            if legacy.team_name == name {
                return self.persist_legacy_team(&legacy).await;
            }
        }
        Ok(None)
    }

    async fn update(&self, mut team: Team) -> Result<Team> {
        team.metadata.touch();
        let data = Self::to_json(&team)?;
        let name = team.name.clone();
        let id = team.id().to_string();
        let sql = format!(
            "UPDATE teams SET name = {}, data = {} WHERE id = {}",
            self.ph(1),
            self.ph(2),
            self.ph(3)
        );
        let stmt = Statement::from_sql_and_values(
            self.backend(),
            &sql,
            [
                Value::String(Some(Box::new(name))),
                Value::String(Some(Box::new(data))),
                Value::String(Some(Box::new(id))),
            ],
        );
        self.db.db.execute(stmt).await.map_err(GatewayError::from)?;
        Ok(team)
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        let id_str = id.to_string();
        let txn = self.db.db.begin().await.map_err(GatewayError::from)?;

        // Remove members before the team row (no DB-level FK constraint).
        let sql = format!("DELETE FROM team_members WHERE team_id = {}", self.ph(1));
        let stmt = Statement::from_sql_and_values(
            self.backend(),
            &sql,
            [Value::String(Some(Box::new(id_str.clone())))],
        );
        txn.execute(stmt).await.map_err(GatewayError::from)?;

        let sql = format!("DELETE FROM teams WHERE id = {}", self.ph(1));
        let stmt = Statement::from_sql_and_values(
            self.backend(),
            &sql,
            [Value::String(Some(Box::new(id_str)))],
        );
        txn.execute(stmt).await.map_err(GatewayError::from)?;

        let sql = format!("DELETE FROM um_teams WHERE team_id = {}", self.ph(1));
        let stmt = Statement::from_sql_and_values(
            self.backend(),
            &sql,
            [Value::String(Some(Box::new(id.to_string())))],
        );
        txn.execute(stmt).await.map_err(GatewayError::from)?;

        txn.commit().await.map_err(GatewayError::from)?;
        Ok(())
    }

    async fn list(&self, offset: u32, limit: u32) -> Result<(Vec<Team>, u64)> {
        self.sync_legacy_um_teams().await?;

        let sql = format!(
            "SELECT data FROM teams WHERE {} ORDER BY created_at ASC LIMIT {} OFFSET {}",
            self.non_deleted_team_predicate(),
            self.ph(1),
            self.ph(2)
        );
        let stmt = Statement::from_sql_and_values(
            self.backend(),
            &sql,
            [
                Value::BigUnsigned(Some(limit as u64)),
                Value::BigUnsigned(Some(offset as u64)),
            ],
        );
        let rows = self
            .db
            .db
            .query_all(stmt)
            .await
            .map_err(GatewayError::from)?;

        let teams: Result<Vec<Team>> = rows
            .into_iter()
            .map(|row| {
                let data: String = row.try_get("", "data").map_err(GatewayError::from)?;
                Self::from_json::<Team>(&data)
            })
            .collect();
        let count_sql = format!(
            "SELECT COUNT(*) as cnt FROM teams WHERE {}",
            self.non_deleted_team_predicate()
        );
        let count_stmt = Statement::from_string(self.backend(), count_sql);
        let total = self
            .db
            .db
            .query_one(count_stmt)
            .await
            .map_err(GatewayError::from)?
            .map(|row| row.try_get::<i64>("", "cnt").unwrap_or(0) as u64)
            .unwrap_or(0);

        Ok((teams?, total))
    }

    async fn count(&self) -> Result<u64> {
        self.sync_legacy_um_teams().await?;

        let sql = format!(
            "SELECT COUNT(*) as cnt FROM teams WHERE {}",
            self.non_deleted_team_predicate()
        );
        let stmt = Statement::from_string(self.backend(), sql);
        Ok(self
            .db
            .db
            .query_one(stmt)
            .await
            .map_err(GatewayError::from)?
            .map(|row| row.try_get::<i64>("", "cnt").unwrap_or(0) as u64)
            .unwrap_or(0))
    }

    async fn add_member(&self, member: TeamMember) -> Result<TeamMember> {
        self.insert_canonical_member(&member).await?;
        Ok(member)
    }

    async fn get_member(&self, team_id: Uuid, user_id: Uuid) -> Result<Option<TeamMember>> {
        if let Some(legacy) = self.get_legacy_um_team(team_id).await? {
            let _ = self.persist_legacy_team(&legacy).await?;
        }

        self.get_canonical_member(team_id, user_id).await
    }

    async fn update_member_role(
        &self,
        team_id: Uuid,
        user_id: Uuid,
        role: TeamRole,
    ) -> Result<TeamMember> {
        let txn = self.db.db.begin().await.map_err(GatewayError::from)?;

        // Read the member inside the transaction to prevent TOCTOU
        let read_sql = format!(
            "SELECT data FROM team_members WHERE team_id = {} AND user_id = {}",
            self.ph(1),
            self.ph(2)
        );
        let read_stmt = Statement::from_sql_and_values(
            self.backend(),
            &read_sql,
            [
                Value::String(Some(Box::new(team_id.to_string()))),
                Value::String(Some(Box::new(user_id.to_string()))),
            ],
        );
        let row = txn
            .query_one(read_stmt)
            .await
            .map_err(GatewayError::from)?
            .ok_or_else(|| {
                GatewayError::NotFound(format!("Member {} not found in team {}", user_id, team_id))
            })?;
        let raw_data: String = row.try_get("", "data").map_err(GatewayError::from)?;
        let mut member: TeamMember = Self::from_json(&raw_data)?;

        member.role = role;
        member.metadata.touch();
        let data = Self::to_json(&member)?;
        let sql = format!(
            "UPDATE team_members SET data = {} WHERE team_id = {} AND user_id = {}",
            self.ph(1),
            self.ph(2),
            self.ph(3)
        );
        let stmt = Statement::from_sql_and_values(
            self.backend(),
            &sql,
            [
                Value::String(Some(Box::new(data))),
                Value::String(Some(Box::new(team_id.to_string()))),
                Value::String(Some(Box::new(user_id.to_string()))),
            ],
        );
        txn.execute(stmt).await.map_err(GatewayError::from)?;

        txn.commit().await.map_err(GatewayError::from)?;
        Ok(member)
    }

    async fn remove_member(&self, team_id: Uuid, user_id: Uuid) -> Result<()> {
        let sql = format!(
            "DELETE FROM team_members WHERE team_id = {} AND user_id = {}",
            self.ph(1),
            self.ph(2)
        );
        let stmt = Statement::from_sql_and_values(
            self.backend(),
            &sql,
            [
                Value::String(Some(Box::new(team_id.to_string()))),
                Value::String(Some(Box::new(user_id.to_string()))),
            ],
        );
        self.db.db.execute(stmt).await.map_err(GatewayError::from)?;
        Ok(())
    }

    async fn list_members(&self, team_id: Uuid) -> Result<Vec<TeamMember>> {
        if let Some(legacy) = self.get_legacy_um_team(team_id).await? {
            let _ = self.persist_legacy_team(&legacy).await?;
        }

        self.list_canonical_members(team_id).await
    }

    async fn get_user_teams(&self, user_id: Uuid) -> Result<Vec<Team>> {
        self.sync_legacy_um_teams().await?;

        let sql = format!(
            "SELECT team_id FROM team_members WHERE user_id = {}",
            self.ph(1)
        );
        let stmt = Statement::from_sql_and_values(
            self.backend(),
            &sql,
            [Value::String(Some(Box::new(user_id.to_string())))],
        );
        let rows = self
            .db
            .db
            .query_all(stmt)
            .await
            .map_err(GatewayError::from)?;
        let mut teams = Vec::new();
        for row in rows {
            let tid: String = row.try_get("", "team_id").map_err(GatewayError::from)?;
            let team_id = tid
                .parse::<Uuid>()
                .map_err(|e| GatewayError::Internal(format!("invalid team uuid {}: {}", tid, e)))?;
            if let Some(team) = self.get(team_id).await? {
                teams.push(team);
            }
        }
        Ok(teams)
    }
}
