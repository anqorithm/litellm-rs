use litellm_contracts::openai::{ChatCompletionRequest, ChatCompletionResponse};

pub async fn handle_completion(
    router: &litellm_router::Router,
    request: ChatCompletionRequest,
) -> Result<ChatCompletionResponse, litellm_router::RouterError> {
    let input = request.user_text_input();

    let result = router.execute(&request.model, &input).await?;

    Ok(ChatCompletionResponse::from_model_output(
        result.model_used,
        result.deployment_id,
        result.output,
    ))
}
