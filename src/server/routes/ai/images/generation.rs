use crate::core::models::openai::{ImageGenerationRequest, ImageGenerationResponse};
use crate::core::pricing_service::PricingUsage;
use crate::core::types::context::RequestContext;
use crate::core::types::image::ImageGenerationRequest as CoreImageRequest;
use crate::core::types::model::ProviderCapability;
use crate::server::state::AppState;
use crate::utils::error::gateway_error::GatewayError;

use super::super::budget_orchestration::{ApiKeyBudgetPolicy, BudgetedCall};
use super::super::execution::execute_with_selected_deployment;

/// Handle image generation with app state (UnifiedRouter only)
pub async fn handle_image_generation_with_state(
    state: &AppState,
    request: ImageGenerationRequest,
    context: RequestContext,
) -> Result<ImageGenerationResponse, GatewayError> {
    let requested_model = request
        .model
        .clone()
        .ok_or_else(|| GatewayError::validation("Model is required"))?;
    if requested_model.trim().is_empty() {
        return Err(GatewayError::validation("Model is required"));
    }

    let core_request = CoreImageRequest {
        prompt: request.prompt,
        model: Some(requested_model.clone()),
        n: request.n,
        size: request.size,
        response_format: request.response_format,
        user: request.user,
        quality: request.quality,
        style: None,
    };

    let context_for_execution = context.clone();
    let api_key_id = context.api_key_id();
    let api_key_budget_id = context.api_key_budget_id();
    let budget_manager = state.budget_manager.clone();
    let budget_limits = state.budget_limits.clone();
    let key_manager = state.key_manager.clone();
    let pricing_service = state.pricing.clone();
    let core_response = execute_with_selected_deployment(
        &state.unified_router,
        &requested_model,
        ProviderCapability::ImageGeneration,
        move |provider, selected_model, _deployment_id| {
            let core_request = core_request.clone();
            let context = context_for_execution.clone();
            let budget_manager = budget_manager.clone();
            let budget_limits = budget_limits.clone();
            let key_manager = key_manager.clone();
            let pricing_service = pricing_service.clone();
            async move {
                let budget_provider = provider.name().to_string();
                let (pricing_provider, mut pricing_model) =
                    super::super::spend::pricing_identity_for_provider(
                        pricing_service.as_ref(),
                        &provider,
                        &selected_model,
                    );
                let mut usage_pricing_model =
                    if super::pricing_keys::is_variant_image_pricing_key(&pricing_model) {
                        selected_model.clone()
                    } else {
                        pricing_model.clone()
                    };
                if let Some(variant_model) = super::pricing_keys::resolve_image_pricing_model(
                    pricing_service.as_ref(),
                    &pricing_provider,
                    &selected_model,
                    core_request.size.as_deref(),
                    core_request.quality.as_deref(),
                )
                .or_else(|| {
                    super::pricing_keys::resolve_image_pricing_model(
                        pricing_service.as_ref(),
                        &pricing_provider,
                        &pricing_model,
                        core_request.size.as_deref(),
                        core_request.quality.as_deref(),
                    )
                }) {
                    usage_pricing_model = variant_model.clone();
                    pricing_model = variant_model;
                }
                let usage = estimated_image_generation_usage(
                    &core_request,
                    &pricing_provider,
                    &usage_pricing_model,
                );
                let mut request_for_provider = core_request.clone();
                request_for_provider.model = Some(selected_model.clone());
                let reserve_pricing_service = pricing_service.clone();
                let settle_pricing_service = pricing_service.clone();
                let reserve_pricing_provider = pricing_provider.clone();
                let reserve_pricing_model = pricing_model.clone();
                let settle_pricing_provider = pricing_provider;
                let settle_pricing_model = pricing_model;
                let reserve_usage = usage.clone();
                let settle_usage = usage;
                let settle_key_manager = key_manager.clone();
                BudgetedCall::new(
                    budget_limits.clone(),
                    budget_provider.clone(),
                    selected_model.clone(),
                )
                    .with_api_key_budget(
                        budget_manager.clone(),
                        api_key_budget_id,
                        ApiKeyBudgetPolicy::FromProviderReservation,
                    )
                    .reserve_call_settle(
                        |budget| {
                            super::super::spend::reserve_pricing_usage_budget_with_pricing(
                                reserve_pricing_service.as_ref(),
                                budget.budget_limits(),
                                budget.provider(),
                                budget.model(),
                                &reserve_pricing_provider,
                                &reserve_pricing_model,
                                &reserve_usage,
                            )
                        },
                        || provider.create_images(request_for_provider, context),
                        |response, reservations, budget| {
                            let (budget_reservation, key_budget_reservation) =
                                reservations.into_parts();
                            async move {
                                let tokens_used = u64::from(
                                    settle_usage
                                        .total_tokens
                                        .saturating_add(settle_usage.image_tokens.unwrap_or(0)),
                                );
                                super::super::spend::record_pricing_usage_spend_with_reservation_with_pricing(
                                    settle_pricing_service.as_ref(),
                                    budget.budget_limits(),
                                    &settle_key_manager,
                                    api_key_id,
                                    budget.provider(),
                                    budget.model(),
                                    &settle_pricing_provider,
                                    &settle_pricing_model,
                                    &settle_usage,
                                    budget_reservation,
                                    key_budget_reservation,
                                )
                                .await;
                                (response, tokens_used)
                            }
                        },
                    )
                    .await
            }
        },
    )
    .await?;

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

fn estimated_image_generation_usage(
    request: &CoreImageRequest,
    pricing_provider: &str,
    pricing_model: &str,
) -> PricingUsage {
    let prompt_tokens = super::estimated_text_tokens(&request.prompt);
    let image_count = request.n.unwrap_or(1);
    let image_tokens = super::estimated_image_output_tokens(
        request.size.as_deref(),
        request.quality.as_deref(),
        image_count,
    );
    let mut usage = PricingUsage::new(prompt_tokens, 0);
    usage.image_tokens = Some(image_tokens);
    usage.output_image_count = Some(image_count.max(1));
    usage.output_image_pricing_keys = super::pricing_keys::image_pricing_keys(
        pricing_provider,
        pricing_model,
        request.size.as_deref(),
        request.quality.as_deref(),
    );
    usage
}
