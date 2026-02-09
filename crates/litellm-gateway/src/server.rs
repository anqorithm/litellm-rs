use crate::state::GatewayState;
use actix_web::{App, HttpResponse, HttpServer, web};
use litellm_contracts::openai::{ChatCompletionRequest, ChatCompletionResponse};

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}

async fn chat_completions(
    state: web::Data<GatewayState>,
    request: web::Json<ChatCompletionRequest>,
) -> HttpResponse {
    let input = request.user_text_input();

    match state.router.execute(&request.model, &input).await {
        Ok(result) => HttpResponse::Ok().json(ChatCompletionResponse::from_model_output(
            result.model_used,
            result.deployment_id,
            result.output,
        )),
        Err(err) => HttpResponse::BadGateway().json(serde_json::json!({
            "error": {
                "message": err.to_string(),
                "type": "gateway_error"
            }
        })),
    }
}

pub async fn run_server(
    config: &litellm_contracts::GatewayConfig,
    state: GatewayState,
) -> anyhow::Result<()> {
    let addr = format!("{}:{}", config.host, config.port);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .route("/health", web::get().to(health))
            .route("/v1/chat/completions", web::post().to(chat_completions))
    })
    .bind(&addr)?
    .run()
    .await?;

    Ok(())
}
