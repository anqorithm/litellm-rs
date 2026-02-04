//! Image generation endpoint

use crate::core::models::RequestContext;
use crate::core::models::openai::{ImageGenerationRequest, ImageGenerationResponse};
use crate::core::providers::ProviderRegistry;
use crate::core::types::ImageGenerationRequest as CoreImageRequest;
use crate::core::types::ProviderCapability;
use crate::server::routes::errors;
use crate::server::state::AppState;
use crate::utils::error::GatewayError;
use actix_web::{HttpRequest, HttpResponse, Result as ActixResult, web};
use tracing::{error, info};

use super::context::{get_request_context, to_core_context};
use super::provider_selection::select_provider_for_optional_model;

/// Image generation endpoint
///
/// OpenAI-compatible image generation API.
pub async fn image_generations(
    state: web::Data<AppState>,
    req: HttpRequest,
    request: web::Json<ImageGenerationRequest>,
) -> ActixResult<HttpResponse> {
    info!("Image generation request for model: {:?}", request.model);

    // Get request context from middleware
    let context = get_request_context(&req)?;

    // Route request through the core router
    match handle_image_generation_via_pool(&state.router, request.into_inner(), context).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => {
            error!("Image generation error: {}", e);
            Ok(errors::gateway_error_to_response(e))
        }
    }
}

/// Handle image generation via provider pool
pub async fn handle_image_generation_via_pool(
    pool: &ProviderRegistry,
    request: ImageGenerationRequest,
    context: RequestContext,
) -> Result<ImageGenerationResponse, GatewayError> {
    let selection = select_provider_for_optional_model(
        pool,
        request.model.as_deref(),
        ProviderCapability::ImageGeneration,
    )?;

    let core_request = CoreImageRequest {
        prompt: request.prompt,
        model: selection.1,
        n: request.n,
        size: request.size,
        response_format: request.response_format,
        user: request.user,
        quality: None,
        style: None,
    };

    let core_context = to_core_context(&context);

    let core_response = selection
        .0
        .image_generation(core_request, core_context)
        .await
        .map_err(|e| GatewayError::internal(format!("Image generation error: {}", e)))?;

    // Convert core response to OpenAI format
    let response = ImageGenerationResponse {
        created: core_response.created,
        data: core_response
            .data
            .into_iter()
            .map(|d| crate::core::models::openai::ImageObject {
                url: d.url,
                b64_json: d.b64_json,
            })
            .collect(),
    };

    Ok(response)
}
