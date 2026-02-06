use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ApiKeys::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ApiKeys::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(ApiKeys::Name).string_len(100).not_null())
                    .col(
                        ColumnDef::new(ApiKeys::KeyHash)
                            .string_len(255)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(ApiKeys::KeyPrefix)
                            .string_len(20)
                            .not_null(),
                    )
                    .col(ColumnDef::new(ApiKeys::UserId).uuid().null())
                    .col(ColumnDef::new(ApiKeys::TeamId).uuid().null())
                    .col(ColumnDef::new(ApiKeys::Permissions).json().null())
                    .col(ColumnDef::new(ApiKeys::RateLimits).json().null())
                    .col(ColumnDef::new(ApiKeys::UsageStats).json().null())
                    .col(
                        ColumnDef::new(ApiKeys::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(ColumnDef::new(ApiKeys::ExpiresAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(ApiKeys::LastUsedAt).timestamp_with_time_zone().null())
                    .col(
                        ColumnDef::new(ApiKeys::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(ApiKeys::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(ApiKeys::Version)
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_api_keys_key_hash")
                    .table(ApiKeys::Table)
                    .col(ApiKeys::KeyHash)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_api_keys_user_id")
                    .table(ApiKeys::Table)
                    .col(ApiKeys::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_api_keys_team_id")
                    .table(ApiKeys::Table)
                    .col(ApiKeys::TeamId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_api_keys_is_active")
                    .table(ApiKeys::Table)
                    .col(ApiKeys::IsActive)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ApiKeys::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ApiKeys {
    Table,
    Id,
    Name,
    KeyHash,
    KeyPrefix,
    UserId,
    TeamId,
    Permissions,
    RateLimits,
    UsageStats,
    IsActive,
    ExpiresAt,
    LastUsedAt,
    CreatedAt,
    UpdatedAt,
    Version,
}
