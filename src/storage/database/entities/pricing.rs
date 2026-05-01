//! Pricing database entities

use sea_orm::entity::prelude::*;
use sea_orm::sea_query::StringLen;
use serde::{Deserialize, Serialize};

/// Model pricing entity for database storage
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "model_pricing")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    /// Provider name (openai, anthropic, glm, etc.)
    #[sea_orm(column_type = "String(StringLen::N(50))")]
    pub provider: String,
    /// Model name
    #[sea_orm(column_type = "String(StringLen::N(100))")]
    pub model: String,
    /// Input token cost per 1K tokens
    pub input_cost_per_1k: f64,
    /// Output token cost per 1K tokens  
    pub output_cost_per_1k: f64,
    /// Currency code
    #[sea_orm(column_type = "String(StringLen::N(10))")]
    pub currency: String,
    /// Whether this is the default pricing for unknown models
    pub is_default: bool,
    /// Additional metadata (JSON)
    #[sea_orm(column_type = "Json", nullable)]
    pub metadata: Option<serde_json::Value>,
    /// Data source (config, api, manual)
    #[sea_orm(column_type = "String(StringLen::N(20))", nullable)]
    pub source: Option<String>,
    /// Created timestamp
    pub created_at: DateTimeUtc,
    /// Updated timestamp  
    pub updated_at: DateTimeUtc,
    /// Expiry timestamp (for cached external data)
    pub expires_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::pricing_history::Entity")]
    PricingHistory,
}

impl Related<super::pricing_history::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PricingHistory.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
