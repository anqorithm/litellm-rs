//! Pricing tables migration

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create model_pricing table
        manager
            .create_table(
                Table::create()
                    .table(ModelPricing::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ModelPricing::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ModelPricing::Provider)
                            .string_len(50)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ModelPricing::Model)
                            .string_len(100)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ModelPricing::InputCostPer1k)
                            .double()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ModelPricing::OutputCostPer1k)
                            .double()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ModelPricing::Currency)
                            .string_len(10)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ModelPricing::IsDefault)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(ModelPricing::Metadata).json())
                    .col(ColumnDef::new(ModelPricing::Source).string_len(20))
                    .col(
                        ColumnDef::new(ModelPricing::CreatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ModelPricing::UpdatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ModelPricing::ExpiresAt).timestamp())
                    .to_owned(),
            )
            .await?;

        // Create indexes for model_pricing
        manager
            .create_index(
                Index::create()
                    .name("idx_model_pricing_provider_model")
                    .table(ModelPricing::Table)
                    .col(ModelPricing::Provider)
                    .col(ModelPricing::Model)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_model_pricing_provider_default")
                    .table(ModelPricing::Table)
                    .col(ModelPricing::Provider)
                    .col(ModelPricing::IsDefault)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_model_pricing_expires_at")
                    .table(ModelPricing::Table)
                    .col(ModelPricing::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        // Create pricing_history table
        manager
            .create_table(
                Table::create()
                    .table(PricingHistory::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PricingHistory::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PricingHistory::PricingId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PricingHistory::Provider)
                            .string_len(50)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PricingHistory::Model)
                            .string_len(100)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PricingHistory::OldInputCostPer1k)
                            .double()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PricingHistory::NewInputCostPer1k)
                            .double()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PricingHistory::OldOutputCostPer1k)
                            .double()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PricingHistory::NewOutputCostPer1k)
                            .double()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PricingHistory::ChangeReason).text())
                    .col(ColumnDef::new(PricingHistory::ChangedBy).string_len(50))
                    .col(
                        ColumnDef::new(PricingHistory::CreatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_pricing_history_pricing_id")
                            .from(PricingHistory::Table, PricingHistory::PricingId)
                            .to(ModelPricing::Table, ModelPricing::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for pricing_history
        manager
            .create_index(
                Index::create()
                    .name("idx_pricing_history_pricing_id")
                    .table(PricingHistory::Table)
                    .col(PricingHistory::PricingId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_pricing_history_provider_model")
                    .table(PricingHistory::Table)
                    .col(PricingHistory::Provider)
                    .col(PricingHistory::Model)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_pricing_history_created_at")
                    .table(PricingHistory::Table)
                    .col(PricingHistory::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PricingHistory::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ModelPricing::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum ModelPricing {
    Table,
    Id,
    Provider,
    Model,
    InputCostPer1k,
    OutputCostPer1k,
    Currency,
    IsDefault,
    Metadata,
    Source,
    CreatedAt,
    UpdatedAt,
    ExpiresAt,
}

#[derive(Iden)]
enum PricingHistory {
    Table,
    Id,
    PricingId,
    Provider,
    Model,
    OldInputCostPer1k,
    NewInputCostPer1k,
    OldOutputCostPer1k,
    NewOutputCostPer1k,
    ChangeReason,
    ChangedBy,
    CreatedAt,
}
