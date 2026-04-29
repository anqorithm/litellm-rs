//! Migration: create virtual keys table
//!
//! Stores virtual key domain objects as JSON snapshots while exposing indexed
//! columns for common lookups and budget/usage updates.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(VirtualKeys::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(VirtualKeys::KeyId)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(VirtualKeys::KeyHash)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(VirtualKeys::UserId).string().not_null())
                    .col(ColumnDef::new(VirtualKeys::Data).text().not_null())
                    .col(
                        ColumnDef::new(VirtualKeys::Spend)
                            .double()
                            .not_null()
                            .default(0.0),
                    )
                    .col(
                        ColumnDef::new(VirtualKeys::BudgetResetAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(VirtualKeys::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(VirtualKeys::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_virtual_keys_user_id")
                    .table(VirtualKeys::Table)
                    .col(VirtualKeys::UserId)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_virtual_keys_budget_reset_at")
                    .table(VirtualKeys::Table)
                    .col(VirtualKeys::BudgetResetAt)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(VirtualKeys::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum VirtualKeys {
    Table,
    KeyId,
    KeyHash,
    UserId,
    Data,
    Spend,
    BudgetResetAt,
    IsActive,
    CreatedAt,
}
