//! Pricing history database entity

use sea_orm::entity::prelude::*;
use sea_orm::sea_query::StringLen;
use serde::{Deserialize, Serialize};

/// Pricing history entity for tracking price changes
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "pricing_history")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    /// Reference to model_pricing
    pub pricing_id: i32,
    /// Provider name
    #[sea_orm(column_type = "String(StringLen::N(50))")]
    pub provider: String,
    /// Model name
    #[sea_orm(column_type = "String(StringLen::N(100))")]
    pub model: String,
    /// Previous input cost
    pub old_input_cost_per_1k: f64,
    /// New input cost
    pub new_input_cost_per_1k: f64,
    /// Previous output cost
    pub old_output_cost_per_1k: f64,
    /// New output cost
    pub new_output_cost_per_1k: f64,
    /// Change reason
    #[sea_orm(column_type = "Text", nullable)]
    pub change_reason: Option<String>,
    /// Changed by (user/system)
    #[sea_orm(column_type = "String(StringLen::N(50))", nullable)]
    pub changed_by: Option<String>,
    /// Created timestamp
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::pricing::Entity",
        from = "Column::PricingId",
        to = "super::pricing::Column::Id"
    )]
    ModelPricing,
}

impl Related<super::pricing::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ModelPricing.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
