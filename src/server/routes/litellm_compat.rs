//! Python LiteLLM proxy compatibility endpoints
//!
//! Admin/ops routes served by the Python proxy that existing tooling
//! (dashboards, Postman collections, uptime probes) calls directly:
//! `/model/info`, `/global/spend`, and `/global/spend/models`. The health
//! aliases live in `routes::health`. Shapes follow the Python proxy closely
//! enough for drop-in consumers; spend figures come from the in-process
//! budget tracker, so they cover the current tracking window rather than
//! all-time database history.

use crate::core::budget::UnifiedBudgetLimits;
use crate::server::state::AppState;
use actix_web::{HttpResponse, Result as ActixResult, web};
use serde::Serialize;
use std::sync::Arc;
use tracing::debug;

/// One entry in the `/model/info` listing
#[derive(Debug, Clone, Serialize)]
pub struct ModelInfoEntry {
    /// Configured (requested) model alias
    pub model_name: String,
    /// Provider-side parameters for the deployment backing the alias
    pub litellm_params: ModelInfoParams,
    /// Deployment metadata
    pub model_info: ModelInfoMeta,
}

/// Provider-side parameters of a deployment
#[derive(Debug, Clone, Serialize)]
pub struct ModelInfoParams {
    /// Provider-qualified model identifier, e.g. `anthropic/claude-sonnet-4-6`
    pub model: String,
}

/// Deployment metadata for `/model/info`
#[derive(Debug, Clone, Serialize)]
pub struct ModelInfoMeta {
    /// Deployment id
    pub id: String,
    /// Provider name
    pub provider: String,
}

#[derive(Debug, Clone, Serialize)]
struct ModelInfoResponse {
    data: Vec<ModelInfoEntry>,
}

/// GET /model/info — list configured model aliases with their deployments
pub async fn model_info(state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    debug!("Model info requested");
    let router = &state.unified_router;

    let mut data = Vec::new();
    for model_name in router.list_models() {
        for deployment_id in router.get_deployments_for_model(&model_name) {
            if let Some(deployment) = router.get_deployment(&deployment_id) {
                let provider = deployment.provider.name().to_string();
                data.push(ModelInfoEntry {
                    model_name: model_name.clone(),
                    litellm_params: ModelInfoParams {
                        model: format!("{}/{}", provider, deployment.model),
                    },
                    model_info: ModelInfoMeta {
                        id: deployment_id,
                        provider,
                    },
                });
            }
        }
    }
    data.sort_by(|a, b| {
        (a.model_name.as_str(), a.model_info.id.as_str())
            .cmp(&(b.model_name.as_str(), b.model_info.id.as_str()))
    });

    Ok(HttpResponse::Ok().json(ModelInfoResponse { data }))
}

/// Response for `/global/spend`
#[derive(Debug, Clone, Serialize)]
pub struct GlobalSpendResponse {
    /// Total tracked spend across providers
    pub spend: f64,
    /// Total allocated budget, if any budget is configured
    pub max_budget: Option<f64>,
}

/// GET /global/spend — total tracked spend for the current window
pub async fn global_spend(
    budget_limits: web::Data<Arc<UnifiedBudgetLimits>>,
) -> ActixResult<HttpResponse> {
    debug!("Global spend requested");
    let provider_usage = budget_limits.providers.list_provider_usage();

    let spend: f64 = provider_usage.iter().map(|u| u.current_spend).sum();
    let allocated: f64 = provider_usage.iter().map(|u| u.max_budget).sum();

    Ok(HttpResponse::Ok().json(GlobalSpendResponse {
        spend,
        max_budget: (allocated > 0.0).then_some(allocated),
    }))
}

/// One row of `/global/spend/models`
#[derive(Debug, Clone, Serialize)]
pub struct ModelSpendRow {
    /// Model name
    pub model: String,
    /// Tracked spend for the model in the current window
    pub total_spend: f64,
    /// Configured budget for the model
    pub max_budget: f64,
    /// Requests counted against the model
    pub request_count: u64,
}

/// GET /global/spend/models — per-model tracked spend for the current window
pub async fn global_spend_models(
    budget_limits: web::Data<Arc<UnifiedBudgetLimits>>,
) -> ActixResult<HttpResponse> {
    debug!("Global model spend requested");
    let mut rows: Vec<ModelSpendRow> = budget_limits
        .models
        .list_model_usage()
        .into_iter()
        .map(|usage| ModelSpendRow {
            model: usage.model_name,
            total_spend: usage.current_spend,
            max_budget: usage.max_budget,
            request_count: usage.request_count,
        })
        .collect();
    rows.sort_by(|a, b| a.model.cmp(&b.model));

    Ok(HttpResponse::Ok().json(rows))
}

/// Configure Python LiteLLM proxy compatibility routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/model/info", web::get().to(model_info)).service(
        web::scope("/global/spend")
            .route("", web::get().to(global_spend))
            .route("/models", web::get().to(global_spend_models)),
    );
}
