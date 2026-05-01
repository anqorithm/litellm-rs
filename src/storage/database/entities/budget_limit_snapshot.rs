use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Provider/model budget snapshot database model.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "budget_limit_snapshots")]
pub struct Model {
    /// Stable scope key, for example `provider:openai`.
    #[sea_orm(primary_key, auto_increment = false)]
    pub scope_key: String,

    /// Scope kind: `provider` or `model`.
    pub scope_type: String,

    /// Provider or model name.
    pub scope_name: String,

    /// Maximum budget.
    pub max_budget: f64,

    /// Current spend.
    pub current_spend: f64,

    /// Soft limit amount.
    pub soft_limit: f64,

    /// Reset period.
    pub reset_period: String,

    /// Currency code.
    pub currency: String,

    /// Whether the limit is enabled.
    pub enabled: bool,

    /// Request count tracked with the spend window.
    pub request_count: i64,

    /// Last reset timestamp.
    pub last_reset_at: Option<DateTimeWithTimeZone>,

    /// Creation timestamp.
    pub created_at: DateTimeWithTimeZone,

    /// Last update timestamp.
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
