//! Migration: create budget limit snapshots table

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(BudgetLimitSnapshots::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BudgetLimitSnapshots::ScopeKey)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(BudgetLimitSnapshots::ScopeType)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BudgetLimitSnapshots::ScopeName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BudgetLimitSnapshots::MaxBudget)
                            .double()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BudgetLimitSnapshots::CurrentSpend)
                            .double()
                            .not_null()
                            .default(0.0),
                    )
                    .col(
                        ColumnDef::new(BudgetLimitSnapshots::SoftLimit)
                            .double()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BudgetLimitSnapshots::ResetPeriod)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BudgetLimitSnapshots::Currency)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BudgetLimitSnapshots::Enabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(BudgetLimitSnapshots::RequestCount)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(BudgetLimitSnapshots::LastResetAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(BudgetLimitSnapshots::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(BudgetLimitSnapshots::UpdatedAt)
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
                    .name("idx_budget_limit_snapshots_scope")
                    .table(BudgetLimitSnapshots::Table)
                    .col(BudgetLimitSnapshots::ScopeType)
                    .col(BudgetLimitSnapshots::ScopeName)
                    .unique()
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BudgetLimitSnapshots::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum BudgetLimitSnapshots {
    Table,
    ScopeKey,
    ScopeType,
    ScopeName,
    MaxBudget,
    CurrentSpend,
    SoftLimit,
    ResetPeriod,
    Currency,
    Enabled,
    RequestCount,
    LastResetAt,
    CreatedAt,
    UpdatedAt,
}
