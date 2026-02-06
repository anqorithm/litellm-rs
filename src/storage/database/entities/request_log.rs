use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Request log database model
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "request_logs")]
pub struct Model {
    /// Log ID
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// Request ID
    #[sea_orm(column_type = "String(Some(100))", unique)]
    pub request_id: String,
    /// Associated user ID
    pub user_id: Option<Uuid>,
    /// Associated team ID
    pub team_id: Option<Uuid>,
    /// Associated API key ID
    pub api_key_id: Option<Uuid>,
    /// Model used
    #[sea_orm(column_type = "String(Some(100))")]
    pub model: String,
    /// Provider used
    #[sea_orm(column_type = "String(Some(50))")]
    pub provider: String,
    /// Request type
    #[sea_orm(column_type = "String(Some(50))")]
    pub request_type: String,
    /// Status
    #[sea_orm(column_type = "String(Some(20))")]
    pub status: String,
    /// HTTP status code
    pub status_code: i32,
    /// Input tokens
    pub input_tokens: i32,
    /// Output tokens
    pub output_tokens: i32,
    /// Total tokens
    pub total_tokens: i32,
    /// Input cost
    pub input_cost: f64,
    /// Output cost
    pub output_cost: f64,
    /// Total cost
    pub total_cost: f64,
    /// Response time in ms
    pub response_time_ms: i64,
    /// Queue time in ms
    pub queue_time_ms: i64,
    /// Provider time in ms
    pub provider_time_ms: i64,
    /// Cache hit indicator
    pub cache_hit: bool,
    /// Error message
    #[sea_orm(column_type = "Text", nullable)]
    pub error_message: Option<String>,
    /// Created timestamp
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
