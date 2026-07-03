use std::time::Duration;

use actix_web::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use actix_web::{HttpResponse, Result as ActixResult};
use bytes::Bytes;
use futures::StreamExt;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::core::models::openai::{ChatCompletionRequest, StreamOptions};
use crate::core::providers::ProviderError;
use crate::core::streaming::types::Event;
use crate::core::types::{context::RequestContext, model::ProviderCapability};
use crate::server::state::AppState;

use super::super::budget_orchestration::{ApiKeyBudgetPolicy, BudgetedCall};
use super::super::execution::execute_stream_with_selected_deployment;
use super::super::openai_errors;
use super::super::{spend, token_policy};

pub(super) async fn handle_streaming_chat_completion(
    state: &AppState,
    mut request: ChatCompletionRequest,
    context: RequestContext,
) -> ActixResult<HttpResponse> {
    info!(
        "Handling streaming chat completion for model: {}",
        request.model
    );

    let requested_model = request.model.clone();
    let request_for_budget = request.clone();
    let client_requested_usage = request
        .stream_options
        .as_ref()
        .and_then(|options| options.include_usage)
        .unwrap_or(false);
    request
        .stream_options
        .get_or_insert(StreamOptions {
            include_usage: None,
        })
        .include_usage = Some(true);
    let core_request = match super::build_core_chat_request(request, requested_model, true) {
        Ok(req) => req,
        Err(e) => return Ok(openai_errors::gateway_error_response(&e)),
    };

    let requested_model = core_request.model.clone();
    let context_for_execution = context.clone();
    let (budget_limits, pricing_service) = (state.budget_limits.clone(), state.pricing.clone());
    let pricing_config = state.config().gateway.pricing.clone();
    let api_key_id = context.api_key_id();
    let api_key_budget_id = context.api_key_budget_id();
    let budget_manager = state.budget_manager.clone();
    match execute_stream_with_selected_deployment(
        state.unified_router.clone(),
        &requested_model,
        ProviderCapability::ChatCompletionStream,
        move |provider, selected_model, _selected_deployment_id| {
            let core_request = core_request.clone();
            let context = context_for_execution.clone();
            let (budget_limits, pricing_service) = (budget_limits.clone(), pricing_service.clone());
            let budget_manager = budget_manager.clone();
            let request_for_budget = request_for_budget.clone();
            let pricing_config = pricing_config.clone();
            async move {
                let provider_name = provider.name().to_string();
                let (pricing_provider, pricing_model) = spend::pricing_identity_for_provider(
                    pricing_service.as_ref(),
                    &provider,
                    &selected_model,
                );
                let (request_for_provider, request_for_budget) =
                    token_policy::prepare_chat_request_for_provider(
                        context.api_key_max_tokens_per_request(),
                        &provider_name,
                        &selected_model,
                        core_request.clone(),
                        request_for_budget,
                    )?;
                let reserve_pricing_service = pricing_service.clone();
                let reserve_pricing_config = pricing_config.clone();
                let reserve_pricing_provider = pricing_provider.clone();
                let reserve_pricing_model = pricing_model.clone();
                let (stream, reservations) = BudgetedCall::new(
                    budget_limits.clone(),
                    provider_name.clone(),
                    selected_model.clone(),
                )
                .with_api_key_budget(
                    budget_manager.clone(),
                    api_key_budget_id,
                    ApiKeyBudgetPolicy::FromProviderReservation,
                )
                .reserve_call(
                    |budget| {
                        spend::reserve_chat_completion_budget_with_split_pricing(
                            reserve_pricing_service.as_ref(),
                            &reserve_pricing_config,
                            budget.budget_limits(),
                            budget.provider(),
                            budget.model(),
                            &reserve_pricing_provider,
                            &reserve_pricing_model,
                            &request_for_budget,
                        )
                    },
                    || provider.chat_completion_stream(request_for_provider, context),
                )
                .await?;
                let (budget_reservation, key_budget_reservation) = reservations.into_parts();
                Ok((
                    stream,
                    provider_name,
                    selected_model,
                    pricing_provider,
                    pricing_model,
                    budget_reservation,
                    key_budget_reservation,
                ))
            }
        },
    )
    .await
    {
        Ok((
            (
                mut stream,
                served_provider,
                served_model,
                pricing_provider,
                pricing_model,
                mut budget_reservation,
                mut key_budget_reservation,
            ),
            lease,
        )) => {
            let (tx, rx) = mpsc::channel::<Bytes>(8);
            let idle_timeout_secs = state.config.load().gateway.server.stream_idle_timeout;
            let budget_limits = state.budget_limits.clone();
            let (key_manager, pricing_service) = (state.key_manager.clone(), state.pricing.clone());
            let pricing_config = state.config().gateway.pricing.clone();

            tokio::spawn(async move {
                let mut lease = Some(lease);
                let mut tokens_used = 0_u64;
                let mut final_usage = None;
                let mut saw_upstream_output = false;
                macro_rules! settle_after_upstream_output {
                    () => {
                        if final_usage.is_some() || saw_upstream_output {
                            spend::record_stream_disconnect_spend_with_reservation_with_policy(
                                pricing_service.as_ref(),
                                &pricing_config,
                                spend::usage_spend_settlement_with_pricing(
                                    (&budget_limits, &key_manager, api_key_id),
                                    (&served_provider, &served_model, final_usage.as_ref()),
                                    (&pricing_provider, &pricing_model),
                                    budget_reservation.take(),
                                    key_budget_reservation.take(),
                                ),
                            )
                            .await;
                        }
                    };
                }

                loop {
                    let chunk_result = if idle_timeout_secs == 0 {
                        stream.next().await
                    } else {
                        let timeout_dur = Duration::from_secs(idle_timeout_secs);
                        match tokio::time::timeout(timeout_dur, stream.next()).await {
                            Ok(result) => result,
                            Err(_) => {
                                warn!(
                                    "SSE stream idle timeout after {}s, closing connection",
                                    idle_timeout_secs
                                );
                                let error_bytes = super::format_sse_error(
                                    &format!(
                                        "Stream idle timeout: no data received for {}s",
                                        idle_timeout_secs
                                    ),
                                    "server_error",
                                    "timeout",
                                );
                                if tx.send(error_bytes).await.is_err() {
                                    info!("Client disconnected before timeout error could be sent");
                                }
                                if let Some(lease) = lease.take() {
                                    let error = ProviderError::timeout(
                                        "router",
                                        format!("stream idle timeout after {}s", idle_timeout_secs),
                                    );
                                    lease.finish_failure(&error);
                                }
                                settle_after_upstream_output!();
                                return;
                            }
                        }
                    };

                    let Some(chunk_result) = chunk_result else {
                        break;
                    };

                    let bytes = match chunk_result {
                        Ok(chunk) => {
                            saw_upstream_output = true;
                            if let Some(usage) = &chunk.usage {
                                tokens_used = u64::from(usage.total_tokens);
                                final_usage = Some(usage.clone());
                            }
                            let mut chat_chunk = match super::convert_core_chunk_to_streaming(chunk)
                            {
                                Ok(chat_chunk) => chat_chunk,
                                Err(e) => {
                                    error!("Stream chunk conversion error: {}", e);
                                    let (error_type, error_code) =
                                        super::sse_error_classification(&e);
                                    let error_bytes = super::format_sse_error(
                                        &e.to_string(),
                                        error_type,
                                        error_code,
                                    );
                                    if tx.send(error_bytes).await.is_err() {
                                        info!(
                                            "Client disconnected before conversion error could be sent"
                                        );
                                    }
                                    if let Some(lease) = lease.take() {
                                        lease.finish_failure(&e);
                                    }
                                    settle_after_upstream_output!();
                                    return;
                                }
                            };
                            if !client_requested_usage {
                                chat_chunk.usage = None;
                                if chat_chunk.choices.is_empty() {
                                    continue;
                                }
                            }
                            match serde_json::to_string(&chat_chunk) {
                                Ok(json) => {
                                    let event = Event::default().data(&json);
                                    event.to_bytes()
                                }
                                Err(e) => {
                                    error!("Stream serialization error: {}", e);
                                    let error_bytes = super::format_sse_error(
                                        &format!("Serialization error: {}", e),
                                        "server_error",
                                        "internal_error",
                                    );
                                    if tx.send(error_bytes).await.is_err() {
                                        info!(
                                            "Client disconnected before error event could be sent"
                                        );
                                    }
                                    if let Some(lease) = lease.take() {
                                        let error = ProviderError::serialization(
                                            "router",
                                            format!("Serialization error: {}", e),
                                        );
                                        lease.finish_failure(&error);
                                    }
                                    settle_after_upstream_output!();
                                    return;
                                }
                            }
                        }
                        Err(e) => {
                            error!("Stream chunk error: {}", e);
                            let (error_type, error_code) = super::sse_error_classification(&e);
                            let error_bytes =
                                super::format_sse_error(&e.to_string(), error_type, error_code);
                            if tx.send(error_bytes).await.is_err() {
                                info!("Client disconnected before error event could be sent");
                            }
                            if let Some(lease) = lease.take() {
                                lease.finish_failure(&e);
                            }
                            settle_after_upstream_output!();
                            return;
                        }
                    };

                    if tx.send(bytes).await.is_err() {
                        info!("Client disconnected during streaming, cancelling upstream");
                        spend::record_stream_disconnect_spend_with_reservation_with_policy(
                            pricing_service.as_ref(),
                            &pricing_config,
                            spend::usage_spend_settlement_with_pricing(
                                (&budget_limits, &key_manager, api_key_id),
                                (&served_provider, &served_model, final_usage.as_ref()),
                                (&pricing_provider, &pricing_model),
                                budget_reservation.take(),
                                key_budget_reservation.take(),
                            ),
                        )
                        .await;
                        return;
                    }
                }

                let done_event = Event::default().data("[DONE]");
                if tx.send(done_event.to_bytes()).await.is_err() {
                    info!("Client disconnected before [DONE] event could be sent");
                }
                spend::record_finished_stream_spend_with_reservation_with_policy(
                    pricing_service.as_ref(),
                    &pricing_config,
                    spend::StreamSpendSettlement {
                        budget_limits: &budget_limits,
                        key_manager: &key_manager,
                        api_key_id,
                        provider: &served_provider,
                        model: &served_model,
                        pricing_provider: &pricing_provider,
                        pricing_model: &pricing_model,
                        usage: final_usage.as_ref(),
                        saw_upstream_output,
                        budget_reservation: budget_reservation.take(),
                        key_budget_reservation: key_budget_reservation.take(),
                    },
                )
                .await;
                if let Some(lease) = lease.take() {
                    lease.finish_success(tokens_used);
                }
            });

            let sse_stream = tokio_stream::wrappers::ReceiverStream::new(rx)
                .map(Ok::<_, actix_web::error::Error>);

            Ok(HttpResponse::Ok()
                .insert_header((CONTENT_TYPE, "text/event-stream"))
                .insert_header((CACHE_CONTROL, "no-cache"))
                .insert_header(("Connection", "keep-alive"))
                .insert_header(("X-Request-ID", context.request_id.as_str()))
                .streaming(sse_stream))
        }
        Err(e) => {
            error!("Failed to create streaming response: {}", e);
            Ok(openai_errors::gateway_error_response(&e))
        }
    }
}
