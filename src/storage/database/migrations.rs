//! Database migrations
//!
//! This module handles database schema migrations.

use crate::utils::error::gateway_error::{GatewayError, Result};
use sqlx::{PgPool, Row};
use tracing::{debug, info};

/// Migration structure
#[derive(Debug)]
pub struct Migration {
    pub version: i32,
    pub name: String,
    pub up_sql: &'static str,
    pub down_sql: &'static str,
}

/// Run all pending migrations
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    info!("Starting database migrations");

    // Create migrations table if it doesn't exist
    create_migrations_table(pool).await?;

    // Get current migration version
    let current_version = get_current_version(pool).await?;
    debug!("Current migration version: {}", current_version);

    // Get all migrations
    let migrations = get_migrations();
    
    // Run pending migrations
    for migration in migrations {
        if migration.version > current_version {
            info!("Running migration {}: {}", migration.version, migration.name);
            run_migration(pool, &migration).await?;
        }
    }

    info!("Database migrations completed");
    Ok(())
}

/// Create the migrations tracking table
async fn create_migrations_table(pool: &PgPool) -> Result<()> {
    let sql = r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            applied_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )
    "#;

    sqlx::query(sql)
        .execute(pool)
        .await
        .map_err(GatewayError::from)?;

    Ok(())
}

/// Get the current migration version
async fn get_current_version(pool: &PgPool) -> Result<i32> {
    let row = sqlx::query("SELECT COALESCE(MAX(version), 0) as version FROM schema_migrations")
        .fetch_one(pool)
        .await
        .map_err(GatewayError::from)?;

    Ok(row.get("version"))
}

/// Run a single migration
async fn run_migration(pool: &PgPool, migration: &Migration) -> Result<()> {
    let mut tx = pool.begin().await.map_err(GatewayError::from)?;

    // Execute the migration SQL
    sqlx::query(migration.up_sql)
        .execute(&mut *tx)
        .await
        .map_err(GatewayError::from)?;

    // Record the migration
    sqlx::query("INSERT INTO schema_migrations (version, name) VALUES ($1, $2)")
        .bind(migration.version)
        .bind(&migration.name)
        .execute(&mut *tx)
        .await
        .map_err(GatewayError::from)?;

    tx.commit().await.map_err(GatewayError::from)?;

    info!("Migration {} completed successfully", migration.version);
    Ok(())
}

/// Get all migrations in order
fn get_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            name: "create_users_table".to_string(),
            up_sql: r#"
                CREATE TABLE users (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    username VARCHAR(50) UNIQUE NOT NULL,
                    email VARCHAR(255) UNIQUE NOT NULL,
                    display_name VARCHAR(100),
                    password_hash VARCHAR(255) NOT NULL,
                    role VARCHAR(20) NOT NULL DEFAULT 'user',
                    status VARCHAR(20) NOT NULL DEFAULT 'pending',
                    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
                    two_factor_enabled BOOLEAN NOT NULL DEFAULT FALSE,
                    last_login_at TIMESTAMP WITH TIME ZONE,
                    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                    version INTEGER NOT NULL DEFAULT 1
                );

                CREATE INDEX idx_users_username ON users(username);
                CREATE INDEX idx_users_email ON users(email);
                CREATE INDEX idx_users_status ON users(status);
            "#,
            down_sql: "DROP TABLE users;",
        },
        Migration {
            version: 2,
            name: "create_teams_table".to_string(),
            up_sql: r#"
                CREATE TABLE teams (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    name VARCHAR(100) UNIQUE NOT NULL,
                    display_name VARCHAR(200),
                    description TEXT,
                    status VARCHAR(20) NOT NULL DEFAULT 'active',
                    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                    version INTEGER NOT NULL DEFAULT 1
                );

                CREATE INDEX idx_teams_name ON teams(name);
                CREATE INDEX idx_teams_status ON teams(status);
            "#,
            down_sql: "DROP TABLE teams;",
        },
        Migration {
            version: 3,
            name: "create_team_members_table".to_string(),
            up_sql: r#"
                CREATE TABLE team_members (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
                    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                    role VARCHAR(20) NOT NULL DEFAULT 'member',
                    status VARCHAR(20) NOT NULL DEFAULT 'active',
                    joined_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                    invited_by UUID REFERENCES users(id),
                    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                    version INTEGER NOT NULL DEFAULT 1,
                    UNIQUE(team_id, user_id)
                );

                CREATE INDEX idx_team_members_team_id ON team_members(team_id);
                CREATE INDEX idx_team_members_user_id ON team_members(user_id);
                CREATE INDEX idx_team_members_status ON team_members(status);
            "#,
            down_sql: "DROP TABLE team_members;",
        },
        Migration {
            version: 4,
            name: "create_api_keys_table".to_string(),
            up_sql: r#"
                CREATE TABLE api_keys (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    name VARCHAR(100) NOT NULL,
                    key_hash VARCHAR(255) NOT NULL UNIQUE,
                    key_prefix VARCHAR(20) NOT NULL,
                    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
                    team_id UUID REFERENCES teams(id) ON DELETE CASCADE,
                    is_active BOOLEAN NOT NULL DEFAULT TRUE,
                    expires_at TIMESTAMP WITH TIME ZONE,
                    last_used_at TIMESTAMP WITH TIME ZONE,
                    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                    version INTEGER NOT NULL DEFAULT 1
                );

                CREATE INDEX idx_api_keys_key_hash ON api_keys(key_hash);
                CREATE INDEX idx_api_keys_user_id ON api_keys(user_id);
                CREATE INDEX idx_api_keys_team_id ON api_keys(team_id);
                CREATE INDEX idx_api_keys_is_active ON api_keys(is_active);
            "#,
            down_sql: "DROP TABLE api_keys;",
        },
        Migration {
            version: 5,
            name: "create_request_logs_table".to_string(),
            up_sql: r#"
                CREATE TABLE request_logs (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    request_id VARCHAR(100) NOT NULL UNIQUE,
                    user_id UUID REFERENCES users(id),
                    team_id UUID REFERENCES teams(id),
                    api_key_id UUID REFERENCES api_keys(id),
                    model VARCHAR(100) NOT NULL,
                    provider VARCHAR(50) NOT NULL,
                    request_type VARCHAR(50) NOT NULL,
                    status VARCHAR(20) NOT NULL,
                    status_code INTEGER NOT NULL,
                    input_tokens INTEGER NOT NULL DEFAULT 0,
                    output_tokens INTEGER NOT NULL DEFAULT 0,
                    total_tokens INTEGER NOT NULL DEFAULT 0,
                    input_cost DECIMAL(10, 6) NOT NULL DEFAULT 0,
                    output_cost DECIMAL(10, 6) NOT NULL DEFAULT 0,
                    total_cost DECIMAL(10, 6) NOT NULL DEFAULT 0,
                    response_time_ms INTEGER NOT NULL,
                    queue_time_ms INTEGER NOT NULL DEFAULT 0,
                    provider_time_ms INTEGER NOT NULL DEFAULT 0,
                    cache_hit BOOLEAN NOT NULL DEFAULT FALSE,
                    error_message TEXT,
                    client_ip INET,
                    user_agent TEXT,
                    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
                );

                CREATE INDEX idx_request_logs_request_id ON request_logs(request_id);
                CREATE INDEX idx_request_logs_user_id ON request_logs(user_id);
                CREATE INDEX idx_request_logs_team_id ON request_logs(team_id);
                CREATE INDEX idx_request_logs_api_key_id ON request_logs(api_key_id);
                CREATE INDEX idx_request_logs_model ON request_logs(model);
                CREATE INDEX idx_request_logs_provider ON request_logs(provider);
                CREATE INDEX idx_request_logs_status ON request_logs(status);
                CREATE INDEX idx_request_logs_created_at ON request_logs(created_at);
            "#,
            down_sql: "DROP TABLE request_logs;",
        },
        Migration {
            version: 6,
            name: "create_usage_stats_table".to_string(),
            up_sql: r#"
                CREATE TABLE usage_stats (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
                    team_id UUID REFERENCES teams(id) ON DELETE CASCADE,
                    date DATE NOT NULL,
                    requests_count INTEGER NOT NULL DEFAULT 0,
                    tokens_count BIGINT NOT NULL DEFAULT 0,
                    cost DECIMAL(10, 6) NOT NULL DEFAULT 0,
                    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                    UNIQUE(user_id, team_id, date)
                );

                CREATE INDEX idx_usage_stats_user_id ON usage_stats(user_id);
                CREATE INDEX idx_usage_stats_team_id ON usage_stats(team_id);
                CREATE INDEX idx_usage_stats_date ON usage_stats(date);
            "#,
            down_sql: "DROP TABLE usage_stats;",
        },
        Migration {
            version: 7,
            name: "create_provider_health_table".to_string(),
            up_sql: r#"
                CREATE TABLE provider_health (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    provider VARCHAR(50) NOT NULL,
                    status VARCHAR(20) NOT NULL,
                    response_time_ms INTEGER,
                    error_message TEXT,
                    success_rate DECIMAL(5, 4) NOT NULL DEFAULT 1.0,
                    total_requests BIGINT NOT NULL DEFAULT 0,
                    failed_requests BIGINT NOT NULL DEFAULT 0,
                    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
                );

                CREATE INDEX idx_provider_health_provider ON provider_health(provider);
                CREATE INDEX idx_provider_health_status ON provider_health(status);
                CREATE INDEX idx_provider_health_created_at ON provider_health(created_at);
            "#,
            down_sql: "DROP TABLE provider_health;",
        },
        Migration {
            version: 8,
            name: "create_user_sessions_table".to_string(),
            up_sql: r#"
                CREATE TABLE user_sessions (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                    token_hash VARCHAR(255) NOT NULL UNIQUE,
                    session_type VARCHAR(20) NOT NULL DEFAULT 'web',
                    ip_address INET,
                    user_agent TEXT,
                    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
                    last_activity TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                    version INTEGER NOT NULL DEFAULT 1
                );

                CREATE INDEX idx_user_sessions_user_id ON user_sessions(user_id);
                CREATE INDEX idx_user_sessions_token_hash ON user_sessions(token_hash);
                CREATE INDEX idx_user_sessions_expires_at ON user_sessions(expires_at);
            "#,
            down_sql: "DROP TABLE user_sessions;",
        },
        Migration {
            version: 9,
            name: "create_password_reset_tokens_table".to_string(),
            up_sql: r#"
                CREATE TABLE password_reset_tokens (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                    token VARCHAR(255) NOT NULL UNIQUE,
                    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
                    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                    used_at TIMESTAMP WITH TIME ZONE,
                    UNIQUE(user_id)
                );

                CREATE INDEX idx_password_reset_tokens_user_id ON password_reset_tokens(user_id);
                CREATE INDEX idx_password_reset_tokens_token ON password_reset_tokens(token);
                CREATE INDEX idx_password_reset_tokens_expires_at ON password_reset_tokens(expires_at);
            "#,
            down_sql: "DROP TABLE password_reset_tokens;",
        },
    ]
}

/// Rollback to a specific version
pub async fn rollback_to_version(pool: &PgPool, target_version: i32) -> Result<()> {
    info!("Rolling back to version {}", target_version);

    let current_version = get_current_version(pool).await?;
    if target_version >= current_version {
        return Err(GatewayError::Config(
            "Target version must be less than current version".to_string()
        ));
    }

    let migrations = get_migrations();
    
    // Run rollbacks in reverse order
    for migration in migrations.iter().rev() {
        if migration.version > target_version && migration.version <= current_version {
            info!("Rolling back migration {}: {}", migration.version, migration.name);
            rollback_migration(pool, migration).await?;
        }
    }

    info!("Rollback to version {} completed", target_version);
    Ok(())
}

/// Rollback a single migration
async fn rollback_migration(pool: &PgPool, migration: &Migration) -> Result<()> {
    let mut tx = pool.begin().await.map_err(GatewayError::from)?;

    // Execute the rollback SQL
    sqlx::query(migration.down_sql)
        .execute(&mut *tx)
        .await
        .map_err(GatewayError::from)?;

    // Remove the migration record
    sqlx::query("DELETE FROM schema_migrations WHERE version = $1")
        .bind(migration.version)
        .execute(&mut *tx)
        .await
        .map_err(GatewayError::from)?;

    tx.commit().await.map_err(GatewayError::from)?;

    info!("Migration {} rolled back successfully", migration.version);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_order() {
        let migrations = get_migrations();
        
        // Check that migrations are in order
        for i in 1..migrations.len() {
            assert!(migrations[i].version > migrations[i-1].version);
        }
        
        // Check that first migration is version 1
        assert_eq!(migrations[0].version, 1);
    }

    #[test]
    fn test_migration_names() {
        let migrations = get_migrations();
        
        // Check that all migrations have names
        for migration in migrations {
            assert!(!migration.name.is_empty());
            assert!(!migration.up_sql.is_empty());
            assert!(!migration.down_sql.is_empty());
        }
    }
}
