use crate::core::budget::UnifiedBudgetLimits;
use crate::core::keys::KeyManager;
use crate::core::pricing_service::PricingService;
use crate::core::pricing_service::PricingUsage;
use crate::core::providers::ProviderError;

const ESTIMATED_AUDIO_BYTES_PER_SECOND: usize = 16_000;

pub(super) fn speech_usage(input: &str) -> PricingUsage {
    let tokens = estimated_audio_text_tokens(input);
    PricingUsage::new(tokens, tokens)
}

pub(super) fn audio_file_usage(file: &[u8], prompt: Option<&str>) -> PricingUsage {
    let file_tokens = u32::try_from(file.len().div_ceil(4))
        .unwrap_or(u32::MAX)
        .max(1);
    let prompt_tokens = prompt.map(estimated_audio_text_tokens).unwrap_or(0);
    let mut usage = PricingUsage::new(file_tokens.saturating_add(prompt_tokens), 0);
    usage.audio_tokens = Some(file_tokens);
    usage
}

pub(super) fn estimated_audio_file_seconds(file: &[u8]) -> f64 {
    file.len().max(1).div_ceil(ESTIMATED_AUDIO_BYTES_PER_SECOND) as f64
}

pub(super) fn reserve_audio_budget(
    budget_limits: &UnifiedBudgetLimits,
    provider: &str,
    model: &str,
    _usage: &PricingUsage,
) -> Result<(), ProviderError> {
    super::super::spend::ensure_budget_available(budget_limits, provider, model)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn record_audio_spend(
    pricing_service: &PricingService,
    budget_limits: &UnifiedBudgetLimits,
    key_manager: &KeyManager,
    api_key_id: Option<uuid::Uuid>,
    budget_provider: &str,
    budget_model: &str,
    pricing_provider: &str,
    pricing_model: &str,
    total_time_seconds: Option<f64>,
    usage: &PricingUsage,
) {
    if let Some(total_time_seconds) = total_time_seconds
        && pricing_service
            .get_model_info_for_provider(pricing_provider, pricing_model)
            .map(|(_, model_info)| model_info.cost_per_second.is_some())
            .unwrap_or(false)
    {
        match pricing_service.calculate_loaded_completion_cost_for_provider(
            pricing_provider,
            pricing_model,
            0,
            0,
            None,
            None,
            Some(total_time_seconds),
        ) {
            Ok(cost) => {
                budget_limits.record_spend(budget_provider, budget_model, cost.total_cost);
                record_key_usage(key_manager, api_key_id, usage, cost.total_cost).await;
            }
            Err(error) => {
                tracing::error!(
                    "time-based audio spend calculation failed for pricing provider \
                     '{pricing_provider}' budget provider '{budget_provider}' model \
                     '{budget_model}': {error}; skipping budget spend"
                );
                record_key_usage(key_manager, api_key_id, usage, 0.0).await;
            }
        }
        return;
    }

    super::super::spend::record_pricing_usage_spend_with_reservation_with_pricing(
        pricing_service,
        budget_limits,
        key_manager,
        api_key_id,
        budget_provider,
        budget_model,
        pricing_provider,
        pricing_model,
        usage,
        None,
    )
    .await;
}

async fn record_key_usage(
    key_manager: &KeyManager,
    api_key_id: Option<uuid::Uuid>,
    usage: &PricingUsage,
    cost: f64,
) {
    if let Some(key_id) = api_key_id {
        let total_tokens = usage
            .total_tokens
            .saturating_add(usage.audio_tokens.unwrap_or(0));
        if let Err(error) = key_manager
            .record_usage(key_id, u64::from(total_tokens), cost)
            .await
        {
            tracing::error!("failed to record usage for key {key_id}: {error}");
        }
    }
}

fn estimated_audio_text_tokens(text: &str) -> u32 {
    u32::try_from(text.chars().count().div_ceil(4))
        .unwrap_or(u32::MAX)
        .max(1)
}
