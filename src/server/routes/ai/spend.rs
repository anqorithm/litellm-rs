//! Spend and usage recording for completed requests.
//!
//! Wires the otherwise-dead budget and per-key usage tracking into the request
//! path: once a completion succeeds and its token usage is known, the served
//! provider/model budget spend and the calling key's usage are recorded.

use uuid::Uuid;

use crate::core::budget::{BudgetReservationError, UnifiedBudgetLimits, UnifiedBudgetReservation};
use crate::core::cost::calculator::{estimate_cost, generic_cost_per_token};
use crate::core::cost::types::UsageTokens;
use crate::core::keys::KeyManager;
use crate::core::models::openai::requests::ChatCompletionRequest;
use crate::core::models::openai::{
    ChatMessage, ContentPart, Function, FunctionCall, MessageContent, ResponseFormat, Tool,
};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::responses::Usage;
use crate::utils::ai::counter::token_counter::TokenCounter;

const IMAGE_PROMPT_BASE_TOKENS: u32 = 85;
const IMAGE_HIGH_DETAIL_PROMPT_TOKENS: u32 = 1_105;
const AUDIO_PROMPT_BASE_TOKENS: u32 = 100;
const DOCUMENT_PROMPT_BASE_TOKENS: u32 = 1_000;

/// Reject a request before it reaches the upstream provider when the served
/// provider or model budget is already exhausted.
///
/// No-ops when budgets are disabled or unconfigured (the availability checks
/// return true). Returns a non-retryable `QuotaExceeded` error (HTTP 402) so
/// the router does not pointlessly retry an over-budget request.
pub(super) fn ensure_budget_available(
    budget_limits: &UnifiedBudgetLimits,
    provider: &str,
    model: &str,
) -> Result<(), ProviderError> {
    if !budget_limits.is_provider_available(provider) {
        return Err(ProviderError::quota_exceeded(
            "budget",
            format!("provider '{provider}' budget exceeded"),
        ));
    }
    if !budget_limits.is_model_available(model) {
        return Err(ProviderError::quota_exceeded(
            "budget",
            format!("model '{model}' budget exceeded"),
        ));
    }
    Ok(())
}

pub(super) fn reserve_completion_budget(
    budget_limits: &UnifiedBudgetLimits,
    provider: &str,
    model: &str,
    estimated_prompt_tokens: u32,
    max_output_tokens: Option<u32>,
) -> Result<Option<UnifiedBudgetReservation>, ProviderError> {
    let estimate = match estimate_cost(model, provider, estimated_prompt_tokens, max_output_tokens)
    {
        Ok(estimate) => estimate,
        Err(e) => {
            tracing::error!(
                "cost estimation failed for '{provider}'/'{model}': {e}; \
                 checking exhausted status without reservation"
            );
            ensure_budget_available(budget_limits, provider, model)?;
            return Ok(None);
        }
    };

    if estimate.max_cost <= 0.0 {
        ensure_budget_available(budget_limits, provider, model)?;
        return Ok(None);
    }

    budget_limits
        .reserve_spend(provider, model, estimate.max_cost)
        .map(Some)
        .map_err(|error| reservation_error_to_provider_error(error, provider, model))
}

pub(super) fn reserve_chat_completion_budget(
    budget_limits: &UnifiedBudgetLimits,
    provider: &str,
    model: &str,
    request: &ChatCompletionRequest,
) -> Result<Option<UnifiedBudgetReservation>, ProviderError> {
    let prompt_tokens = estimate_chat_prompt_tokens(
        model,
        &request.messages,
        request.tools.as_deref(),
        request.functions.as_deref(),
        request.function_call.as_ref(),
        request.response_format.as_ref(),
    );
    reserve_completion_budget(
        budget_limits,
        provider,
        model,
        prompt_tokens,
        reservation_output_tokens(
            provider,
            model,
            prompt_tokens,
            provider_effective_max_output_tokens(provider, model, request),
            request.n.unwrap_or(1),
        ),
    )
}

pub(super) fn estimate_chat_prompt_tokens(
    model: &str,
    messages: &[ChatMessage],
    tools: Option<&[Tool]>,
    functions: Option<&[Function]>,
    function_call: Option<&FunctionCall>,
    response_format: Option<&ResponseFormat>,
) -> u32 {
    let counter = TokenCounter::new();
    let message_tokens = match counter.count_chat_tokens(model, messages) {
        Ok(estimate) => estimate.input_tokens,
        Err(error) => {
            tracing::warn!(
                "token estimation failed for model '{model}': {error}; using fallback estimate"
            );
            fallback_message_tokens(messages)
        }
    };
    let multimodal_tokens = conservative_multimodal_prompt_extra(messages);

    let tool_tokens = tools.map_or(0, |tools| {
        let Ok(tool_json) = serde_json::to_string(tools) else {
            return u32::try_from(tools.len().saturating_mul(256)).unwrap_or(u32::MAX);
        };
        counter
            .count_completion_tokens(model, &tool_json)
            .map(|estimate| estimate.input_tokens)
            .unwrap_or_else(|error| {
                tracing::warn!(
                    "tool token estimation failed for model '{model}': {error}; \
                     using fallback estimate"
                );
                u32::try_from(tool_json.chars().count().div_ceil(4)).unwrap_or(u32::MAX)
            })
    });

    let function_tokens = serialized_prompt_tokens(
        &counter,
        model,
        functions,
        "legacy function token estimation failed",
        |functions| functions.len().saturating_mul(256),
    );
    let function_call_tokens = serialized_prompt_tokens(
        &counter,
        model,
        function_call,
        "legacy function_call token estimation failed",
        |_| 64,
    );
    let response_format_tokens = serialized_prompt_tokens(
        &counter,
        model,
        response_format,
        "response_format token estimation failed",
        |_| 128,
    );

    message_tokens
        .saturating_add(multimodal_tokens)
        .saturating_add(tool_tokens)
        .saturating_add(function_tokens)
        .saturating_add(function_call_tokens)
        .saturating_add(response_format_tokens)
}

fn reservation_output_tokens(
    provider: &str,
    model: &str,
    prompt_tokens: u32,
    requested_max_output_tokens: Option<u32>,
    choice_count: u32,
) -> Option<u32> {
    let counter = TokenCounter::new();
    let choice_count = choice_count.max(1);
    let output_tokens = if let Some(requested) = requested_max_output_tokens {
        Some(requested)
    } else {
        catalog_max_output_tokens(provider, model).or_else(|| {
            counter
                .estimate_output_tokens(None, prompt_tokens, model)
                .ok()
        })
    };

    output_tokens.map(|tokens| tokens.saturating_mul(choice_count))
}

fn catalog_max_output_tokens(provider: &str, model: &str) -> Option<u32> {
    let db = crate::core::pricing::get_pricing_db();
    let provider_aliases = pricing_provider_aliases(provider, model);
    if let Some(tokens) = db
        .get_model_info(model)
        .filter(|info| provider_name_matches(&info.litellm_provider, &provider_aliases))
        .and_then(|info| info.max_output_tokens)
    {
        return Some(tokens);
    }

    let normalized_model = crate::core::pricing::normalize_model_key(model);
    if normalized_model != model
        && let Some(tokens) = db
            .get_model_info(normalized_model)
            .filter(|info| provider_name_matches(&info.litellm_provider, &provider_aliases))
            .and_then(|info| info.max_output_tokens)
    {
        return Some(tokens);
    }

    provider_aliases
        .iter()
        .flat_map(|provider| db.get_provider_models(provider))
        .filter(|candidate| model_id_matches(&candidate.to_lowercase(), normalized_model))
        .filter_map(|candidate| db.get_model_info(&candidate))
        .filter(|info| provider_name_matches(&info.litellm_provider, &provider_aliases))
        .filter_map(|info| info.max_output_tokens)
        .max()
}

fn provider_effective_max_output_tokens(
    provider: &str,
    model: &str,
    request: &ChatCompletionRequest,
) -> Option<u32> {
    let provider = crate::core::pricing::normalize_pricing_provider(provider);
    match provider.as_str() {
        "openai" | "azure" | "azure_ai" | "openai_like" | "openrouter" | "xai" | "groq"
        | "deepseek" | "moonshot" | "minimax" | "zhipuai" | "xiaomi_mimo" | "amazon_nova"
        | "ai21" | "baseten" | "huggingface" | "ollama" | "sagemaker" | "snowflake" => {
            request.max_completion_tokens.or(request.max_tokens)
        }
        "bedrock" => bedrock_effective_max_output_tokens(model, request),
        "cohere" | "replicate" => request.max_tokens.or(request.max_completion_tokens),
        _ => request.max_tokens,
    }
}

fn bedrock_effective_max_output_tokens(
    model: &str,
    request: &ChatCompletionRequest,
) -> Option<u32> {
    use crate::core::providers::bedrock::BedrockApiType;

    let Ok(config) = crate::core::providers::bedrock::get_model_config_for_model_id(model) else {
        return request.max_tokens;
    };

    match config.api_type {
        BedrockApiType::Converse | BedrockApiType::ConverseStream => {
            request.max_completion_tokens.or(request.max_tokens)
        }
        BedrockApiType::Invoke | BedrockApiType::InvokeStream => request.max_tokens,
    }
}

fn pricing_provider_aliases(provider: &str, model: &str) -> Vec<String> {
    let normalized = crate::core::pricing::normalize_pricing_provider(provider);
    let aliases = match normalized.as_str() {
        "anthropic" if is_xiaomi_mimo_model(model) => vec!["xiaomi_mimo", "xiaomi", "mimo"],
        "gemini" => vec!["gemini", "vertex_ai"],
        "vertex_ai" => vec!["vertex_ai", "google"],
        "xiaomi_mimo" => vec!["xiaomi_mimo", "xiaomi", "mimo"],
        "zhipuai" => vec!["zhipuai", "glm", "zai"],
        _ => return vec![normalized],
    };
    aliases
        .into_iter()
        .map(crate::core::pricing::normalize_pricing_provider)
        .fold(Vec::new(), |mut unique, alias| {
            if !unique.contains(&alias) {
                unique.push(alias);
            }
            unique
        })
}

fn is_xiaomi_mimo_model(model: &str) -> bool {
    crate::core::pricing::normalize_model_key(model).starts_with("mimo-")
}

fn provider_name_matches(provider: &str, aliases: &[String]) -> bool {
    let provider = crate::core::pricing::normalize_pricing_provider(provider);
    aliases
        .iter()
        .any(|alias| crate::core::pricing::normalize_pricing_provider(alias) == provider)
}

fn model_id_matches(candidate: &str, requested: &str) -> bool {
    candidate == requested
        || has_dash_suffix(candidate, requested)
        || has_dash_suffix(requested, candidate)
}

fn has_dash_suffix(value: &str, prefix: &str) -> bool {
    value
        .strip_prefix(prefix)
        .and_then(|suffix| suffix.strip_prefix('-'))
        .is_some()
}

fn conservative_multimodal_prompt_extra(messages: &[ChatMessage]) -> u32 {
    messages
        .iter()
        .filter_map(|message| message.content.as_ref())
        .flat_map(|content| match content {
            MessageContent::Text(_) => [].as_slice(),
            MessageContent::Parts(parts) => parts.as_slice(),
        })
        .fold(0u32, |total, part| {
            total.saturating_add(conservative_content_part_extra(part))
        })
}

fn conservative_content_part_extra(part: &ContentPart) -> u32 {
    match part {
        ContentPart::ImageUrl { image_url } => {
            image_prompt_floor(image_url.detail.as_deref()).saturating_sub(IMAGE_PROMPT_BASE_TOKENS)
        }
        ContentPart::Image {
            source,
            detail,
            image_url,
        } => {
            let detail = detail
                .as_deref()
                .or_else(|| image_url.as_ref().and_then(|url| url.detail.as_deref()));
            image_prompt_floor(detail)
                .max(encoded_media_tokens(&source.data))
                .saturating_sub(IMAGE_PROMPT_BASE_TOKENS)
        }
        ContentPart::Audio { audio } => encoded_media_tokens(&audio.data)
            .max(AUDIO_PROMPT_BASE_TOKENS)
            .saturating_sub(AUDIO_PROMPT_BASE_TOKENS),
        ContentPart::Document { source, .. } => encoded_media_tokens(&source.data)
            .max(DOCUMENT_PROMPT_BASE_TOKENS)
            .saturating_sub(DOCUMENT_PROMPT_BASE_TOKENS),
        ContentPart::Text { .. } | ContentPart::ToolResult { .. } | ContentPart::ToolUse { .. } => {
            0
        }
    }
}

fn image_prompt_floor(detail: Option<&str>) -> u32 {
    if detail.is_some_and(|detail| detail.eq_ignore_ascii_case("low")) {
        IMAGE_PROMPT_BASE_TOKENS
    } else {
        IMAGE_HIGH_DETAIL_PROMPT_TOKENS
    }
}

fn encoded_media_tokens(data: &str) -> u32 {
    u32::try_from(data.chars().count().div_ceil(4)).unwrap_or(u32::MAX)
}

fn serialized_prompt_tokens<T, F>(
    counter: &TokenCounter,
    model: &str,
    value: Option<&T>,
    warn_message: &str,
    fallback_units: F,
) -> u32
where
    T: serde::Serialize + ?Sized,
    F: FnOnce(&T) -> usize,
{
    let Some(value) = value else {
        return 0;
    };
    let Ok(json) = serde_json::to_string(value) else {
        return u32::try_from(fallback_units(value)).unwrap_or(u32::MAX);
    };

    counter
        .count_completion_tokens(model, &json)
        .map(|estimate| estimate.input_tokens)
        .unwrap_or_else(|error| {
            tracing::warn!("{warn_message} for model '{model}': {error}; using fallback estimate");
            u32::try_from(json.chars().count().div_ceil(4)).unwrap_or(u32::MAX)
        })
}

fn fallback_message_tokens(messages: &[ChatMessage]) -> u32 {
    let chars = messages
        .iter()
        .filter_map(|message| message.content.as_ref())
        .map(|content| match content {
            MessageContent::Text(text) => text.chars().count(),
            MessageContent::Parts(parts) => serde_json::to_string(parts)
                .map(|text| text.chars().count())
                .unwrap_or_default(),
        })
        .sum::<usize>();
    let overhead = messages.len().saturating_mul(4).saturating_add(8);
    u32::try_from(chars.div_ceil(4).saturating_add(overhead)).unwrap_or(u32::MAX)
}

fn reservation_error_to_provider_error(
    error: BudgetReservationError,
    provider: &str,
    model: &str,
) -> ProviderError {
    match error {
        BudgetReservationError::ProviderBudgetExceeded => ProviderError::quota_exceeded(
            "budget",
            format!("provider '{provider}' budget exceeded"),
        ),
        BudgetReservationError::ModelBudgetExceeded => {
            ProviderError::quota_exceeded("budget", format!("model '{model}' budget exceeded"))
        }
        BudgetReservationError::BudgetExceeded => ProviderError::quota_exceeded(
            "budget",
            format!("budget exceeded for provider '{provider}' model '{model}'"),
        ),
        BudgetReservationError::InvalidAmount(error) => ProviderError::invalid_request(
            "budget",
            format!("invalid budget reservation amount for '{provider}'/'{model}': {error}"),
        ),
        BudgetReservationError::ActualExceedsReservation => ProviderError::invalid_request(
            "budget",
            format!("actual spend exceeded reserved budget for '{provider}'/'{model}'"),
        ),
    }
}

/// Record provider/model budget spend and per-key usage for a completed request.
///
/// Best-effort and non-fatal: the completion already succeeded, so failures here
/// are logged at error level (never silently swallowed) but do not fail the
/// response. When the cost cannot be priced, token usage is still recorded but
/// budget spend is skipped rather than booked at $0 — under-counting a budget is
/// worse than leaving it unchanged with a loud error.
pub(super) async fn record_completion_spend_with_reservation(
    budget_limits: &UnifiedBudgetLimits,
    key_manager: &KeyManager,
    api_key_id: Option<Uuid>,
    provider: &str,
    model: &str,
    usage: Option<&Usage>,
    budget_reservation: Option<UnifiedBudgetReservation>,
) {
    let Some(usage) = usage else {
        tracing::error!(
            "provider '{provider}' returned no usage for model '{model}'; spend not recorded"
        );
        return;
    };

    let total_tokens = u64::from(usage.total_tokens);
    let usage_tokens: UsageTokens = usage.clone().into();

    let cost = match generic_cost_per_token(model, &usage_tokens, provider) {
        Ok(breakdown) => Some(breakdown.total_cost),
        Err(e) => {
            tracing::error!(
                "cost calculation failed for '{provider}'/'{model}': {e}; \
                 recording token usage without cost and skipping budget spend"
            );
            None
        }
    };

    if let Some(cost) = cost {
        if let Some(reservation) = budget_reservation {
            if let Err(error) = reservation.settle(cost) {
                tracing::error!(
                    "failed to settle reserved budget for '{provider}'/'{model}': {error:?}; \
                     spend not recorded because reservation settlement failed"
                );
            }
        } else {
            budget_limits.record_spend(provider, model, cost);
        }
    }

    if let Some(key_id) = api_key_id {
        // Token counts are factual even when pricing is unavailable; record them
        // with the cost we have (0.0 only when pricing failed, already logged).
        if let Err(e) = key_manager
            .record_usage(key_id, total_tokens, cost.unwrap_or(0.0))
            .await
        {
            tracing::error!("failed to record usage for key {key_id}: {e}");
        }
    }
}

pub(super) async fn record_stream_disconnect_spend_with_reservation(
    budget_limits: &UnifiedBudgetLimits,
    key_manager: &KeyManager,
    api_key_id: Option<Uuid>,
    provider: &str,
    model: &str,
    usage: Option<&Usage>,
    budget_reservation: Option<UnifiedBudgetReservation>,
) {
    if let Some(usage) = usage {
        record_completion_spend_with_reservation(
            budget_limits,
            key_manager,
            api_key_id,
            provider,
            model,
            Some(usage),
            budget_reservation,
        )
        .await;
        return;
    }

    let Some(reservation) = budget_reservation else {
        tracing::error!(
            "client disconnected before provider '{provider}' returned usage for model '{model}'; spend not recorded"
        );
        return;
    };
    let reserved = reservation.reserved_amount();
    if let Err(error) = reservation.settle(reserved) {
        tracing::error!(
            "failed to settle reserved budget after stream disconnect for '{provider}'/'{model}': {error:?}"
        );
    }
}

#[cfg(test)]
#[path = "spend_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "spend_provider_reservation_tests.rs"]
mod provider_reservation_tests;
