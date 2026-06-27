use super::*;
use crate::core::budget::{ProviderLimitConfig, ResetPeriod};
use crate::core::cost::calculator::estimate_cost;
use crate::core::models::openai::requests::ChatCompletionRequest;
use crate::core::models::openai::{ChatMessage, MessageContent, MessageRole};

fn reserve_with_provider_limit(
    provider: &str,
    model: &str,
    max_output_tokens: u32,
) -> UnifiedBudgetReservation {
    let budget = UnifiedBudgetLimits::new();
    let messages = vec![ChatMessage {
        role: MessageRole::User,
        content: Some(MessageContent::Text("hello".to_string())),
        name: None,
        function_call: None,
        tool_calls: None,
        tool_call_id: None,
        audio: None,
    }];
    let prompt_tokens = estimate_chat_prompt_tokens(model, &messages, None, None, None, None);
    let estimate = estimate_cost(model, provider, prompt_tokens, Some(max_output_tokens)).unwrap();
    budget.providers.set_provider_limit(
        provider,
        ProviderLimitConfig::new(estimate.max_cost * 2.0, ResetPeriod::Monthly),
    );

    let mut request = ChatCompletionRequest {
        model: model.to_string(),
        messages,
        ..Default::default()
    };
    request.max_completion_tokens = Some(max_output_tokens);
    if provider == "bedrock" {
        request.max_tokens = Some(max_output_tokens);
    }

    let reservation = reserve_chat_completion_budget(&budget, provider, model, &request)
        .unwrap()
        .unwrap();
    assert!((reservation.reserved_amount() - estimate.max_cost).abs() < f64::EPSILON);
    reservation
}

#[test]
fn bedrock_chat_reservation_uses_bedrock_cost_pricing() {
    let reservation = reserve_with_provider_limit("bedrock", "amazon.titan-text-express-v1", 100);
    reservation.cancel();
}

#[test]
fn amazon_nova_chat_reservation_uses_provider_pricing() {
    let reservation = reserve_with_provider_limit("amazon_nova", "amazon.nova-2-lite-v1:0", 10);
    reservation.cancel();
}

#[test]
fn openai_like_prefixed_chat_reservation_uses_provider_pricing() {
    let reservation =
        reserve_with_provider_limit("openai_like", "groq/llama-3.3-70b-versatile", 100);
    reservation.cancel();
}
