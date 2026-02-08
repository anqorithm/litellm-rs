//! Model listing and retrieval endpoints

use crate::core::models::openai::{Model, ModelListResponse};
use crate::core::providers::ProviderRegistry;
use crate::server::routes::ApiResponse;
use crate::server::state::AppState;
use crate::utils::error::error::GatewayError;
use actix_web::{HttpResponse, Result as ActixResult, web};
use tracing::{debug, error};

/// List available models
///
/// Returns a list of available AI models across all configured providers.
pub async fn list_models(state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    debug!("Listing available models");

    // TODO: Implement proper model listing through ProviderRegistry
    match get_models_from_pool(&state.router).await {
        Ok(models) => {
            let response = ModelListResponse {
                object: "list".to_string(),
                data: models,
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            error!("Failed to list models: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Error".to_string())))
        }
    }
}

/// Get specific model information
///
/// Returns detailed information about a specific model.
pub async fn get_model(
    state: web::Data<AppState>,
    model_id: web::Path<String>,
) -> ActixResult<HttpResponse> {
    debug!("Getting model info for: {}", model_id);

    // TODO: Implement proper model retrieval through ProviderRegistry
    match get_model_from_pool(&state.router, &model_id).await {
        Ok(Some(model)) => Ok(HttpResponse::Ok().json(model)),
        Ok(None) => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Error".to_string())))
        }
        Err(e) => {
            error!("Failed to get model {}: {}", model_id, e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Error".to_string())))
        }
    }
}

/// Get all models from provider pool
pub async fn get_models_from_pool(pool: &ProviderRegistry) -> Result<Vec<Model>, GatewayError> {
    let mut all_models = Vec::new();

    // Get models from all providers
    let providers = pool.get_all_providers();
    for provider in providers {
        let models = provider.list_models();
        for model_info in models {
            all_models.push(Model {
                id: model_info.id.clone(),
                object: "model".to_string(),
                created: chrono::Utc::now().timestamp() as u64,
                owned_by: model_info.provider.clone(),
            });
        }
    }

    Ok(all_models)
}

/// Get specific model from provider pool
pub async fn get_model_from_pool(
    _pool: &ProviderRegistry,
    _model_id: &str,
) -> Result<Option<Model>, GatewayError> {
    // TODO: Get specific model from providers in pool
    Ok(None) // Return None for now
}
