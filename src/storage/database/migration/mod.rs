use sea_orm_migration::prelude::*;

mod m20240101_000001_create_users_table;
mod m20240101_000002_create_password_reset_tokens_table;
mod m20240101_000003_create_batches_table;
mod m20240101_000004_create_user_sessions_table;
mod m20240205_000001_create_api_keys_table;
mod m20240205_000002_create_request_logs_table;

/// Database migrator for SeaORM
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_users_table::Migration),
            Box::new(m20240101_000002_create_password_reset_tokens_table::Migration),
            Box::new(m20240101_000003_create_batches_table::Migration),
            Box::new(m20240101_000004_create_user_sessions_table::Migration),
            Box::new(m20240205_000001_create_api_keys_table::Migration),
            Box::new(m20240205_000002_create_request_logs_table::Migration),
        ]
    }
}
