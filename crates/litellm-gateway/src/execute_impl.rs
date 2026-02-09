use litellm_contracts::openai::ChatCompletionResponse;

pub async fn execute_sample(
    state: &crate::state::GatewayState,
    model: &str,
    input: &str,
) -> anyhow::Result<ChatCompletionResponse> {
    let result = state
        .router
        .execute(model, input)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    Ok(ChatCompletionResponse::from_model_output(
        result.model_used,
        result.deployment_id,
        result.output,
    ))
}
