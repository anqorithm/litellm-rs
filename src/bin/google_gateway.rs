//! Configuration
//!
//! Configuration

use actix_web::{
    App, HttpResponse, HttpServer, Result as ActixResult,
    middleware::{DefaultHeaders, Logger},
    web,
};

use actix_cors::Cors;

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{error, info, instrument};

/// Configuration
#[derive(Debug, Clone, Deserialize)]
pub struct GatewayConfig {
    pub server: ServerConfig,
    pub google: GoogleConfig,
    pub model_mapping: HashMap<String, String>,
    pub logging: LoggingConfig,
    pub security: SecurityConfig,
    pub monitoring: MonitoringConfig,
    pub cache: CacheConfig,
}

/// Configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout: u64,
    pub max_body_size: usize,
}

/// Configuration
#[derive(Debug, Clone, Deserialize)]
pub struct GoogleConfig {
    pub api_key: String,
    pub base_url: String,
    pub timeout: u64,
    pub max_retries: u32,
    pub models: Vec<ModelConfig>,
}

/// Configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub google_model: String,
    pub max_tokens: u32,
    pub enabled: bool,
}

/// Configuration
#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub show_request_body: bool,
    pub show_response_body: bool,
}

/// Configuration
#[derive(Debug, Clone, Deserialize)]
pub struct SecurityConfig {
    pub cors_enabled: bool,
    pub rate_limit: RateLimitConfig,
}

/// Configuration
#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub requests_per_minute: u32,
}

/// Configuration
#[derive(Debug, Clone, Deserialize)]
pub struct MonitoringConfig {
    pub health_check: bool,
    pub metrics: bool,
    pub request_logging: bool,
}

/// Configuration
#[derive(Debug, Clone, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub ttl_seconds: u64,
    pub max_size: usize,
}

/// Application state
#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Arc<GatewayConfig>,
    /// Request count - using AtomicU64 for lock-free access
    pub request_count: Arc<AtomicU64>,
    pub http_client: reqwest::Client,
}

/// Chat completion request
#[derive(Debug, Deserialize)]
pub struct GoogleChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
}

/// Message structure
#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Google API request structure
#[derive(Debug, Serialize)]
pub struct GoogleRequest {
    pub contents: Vec<GoogleContent>,
    #[serde(rename = "generationConfig")]
    pub generation_config: GoogleGenerationConfig,
}

#[derive(Debug, Serialize)]
pub struct GoogleContent {
    pub parts: Vec<GooglePart>,
}

#[derive(Debug, Serialize)]
pub struct GooglePart {
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct GoogleGenerationConfig {
    pub temperature: f32,
    #[serde(rename = "maxOutputTokens")]
    pub max_output_tokens: u32,
}

/// Google API response
#[derive(Debug, Deserialize)]
pub struct GoogleResponse {
    pub candidates: Vec<GoogleCandidate>,
}

#[derive(Debug, Deserialize)]
pub struct GoogleCandidate {
    pub content: GoogleResponseContent,
}

#[derive(Debug, Deserialize)]
pub struct GoogleResponseContent {
    pub parts: Vec<GoogleResponsePart>,
}

#[derive(Debug, Deserialize)]
pub struct GoogleResponsePart {
    pub text: String,
}

/// Check
#[instrument(skip(state))]
async fn health_check(state: web::Data<AppState>) -> HttpResponse {
    let count = state.request_count.fetch_add(1, Ordering::Relaxed) + 1;

    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "service": "Google API Gateway",
        "version": "1.0.0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "requests_served": count
    }))
}

/// Model
#[instrument(skip(state))]
async fn list_models(state: web::Data<AppState>) -> HttpResponse {
    state.request_count.fetch_add(1, Ordering::Relaxed);

    HttpResponse::Ok().json(json!({
        "object": "list",
        "data": [
            {
                "id": "gemini-1.5-pro",
                "object": "model",
                "created": 1677610602,
                "owned_by": "google"
            },
            {
                "id": "gemini-1.5-flash",
                "object": "model",
                "created": 1677610602,
                "owned_by": "google"
            },
            {
                "id": "gemini-pro",
                "object": "model",
                "created": 1677610602,
                "owned_by": "google"
            }
        ]
    }))
}

/// Chat completion - actual Google API call
#[instrument(skip(state))]
async fn chat_completions(
    state: web::Data<AppState>,
    request: web::Json<GoogleChatRequest>,
) -> ActixResult<HttpResponse> {
    info!(
        "🤖 Processing actual Google API request: model={}",
        request.model
    );

    state.request_count.fetch_add(1, Ordering::Relaxed);

    // Check
    let requested_model = &request.model;
    let mapped_model = state
        .config
        .model_mapping
        .get(requested_model)
        .unwrap_or(requested_model);

    // Configuration
    let model_config = state
        .config
        .google
        .models
        .iter()
        .find(|m| m.name == *mapped_model && m.enabled)
        .ok_or_else(|| {
            error!("❌ Model unavailable: {}", mapped_model);
            actix_web::error::ErrorBadRequest("Model not available")
        })?;

    info!(
        "📋 Using model: {} -> {}",
        requested_model, model_config.google_model
    );

    // Convert to Google API format
    let google_request = GoogleRequest {
        contents: request
            .messages
            .iter()
            .map(|msg| GoogleContent {
                parts: vec![GooglePart {
                    text: msg.content.clone(),
                }],
            })
            .collect(),
        generation_config: GoogleGenerationConfig {
            temperature: request.temperature.unwrap_or(0.7),
            max_output_tokens: request
                .max_tokens
                .unwrap_or(model_config.max_tokens)
                .min(model_config.max_tokens),
        },
    };

    // Build
    let url = format!(
        "{}/models/{}:generateContent",
        state.config.google.base_url, model_config.google_model
    );

    info!("📡 callGoogle API: {}", url);

    // callGoogle API
    let response = state
        .http_client
        .post(&url)
        .header("Content-Type", "application/json")
        .query(&[("key", &state.config.google.api_key)])
        .json(&google_request)
        .timeout(std::time::Duration::from_secs(state.config.google.timeout))
        .send()
        .await
        .map_err(|e| {
            error!("❌ Google API request failed: {}", e);
            actix_web::error::ErrorInternalServerError("Google API request failed")
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        error!("❌ Google API returned error {}: {}", status, error_text);
        return Ok(HttpResponse::BadGateway().json(json!({
            "error": "Google API request failed"
        })));
    }

    let google_response: GoogleResponse = response.json().await.map_err(|e| {
        error!("❌ Failed to parse Google API response: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to parse Google API response")
    })?;

    // Convert to OpenAI format
    let content = google_response
        .candidates
        .first()
        .and_then(|c| c.content.parts.first())
        .map(|p| p.text.clone())
        .unwrap_or_else(|| "Sorry, unable to generate response.".to_string());

    info!(
        "✅ Google API response successful, content length: {}",
        content.len()
    );

    let openai_response = json!({
        "id": format!("chatcmpl-{}", uuid::Uuid::new_v4()),
        "object": "chat.completion",
        "created": chrono::Utc::now().timestamp(),
        "model": requested_model,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": content
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 20,
            "completion_tokens": content.len() / 4, // Rough estimate
            "total_tokens": 20 + content.len() / 4
        }
    });

    Ok(HttpResponse::Ok().json(openai_response))
}

/// Configuration
pub struct ConfigurableGateway {
    config: GatewayConfig,
}

impl ConfigurableGateway {
    pub async fn new(
        config: GatewayConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self { config })
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let state = AppState {
            config: Arc::new(self.config.clone()),
            request_count: Arc::new(AtomicU64::new(0)),
            http_client: reqwest::Client::new(),
        };

        let bind_addr = format!("{}:{}", self.config.server.host, self.config.server.port);
        let state_data = web::Data::new(state);

        let cors_enabled = self.config.security.cors_enabled;
        let server = HttpServer::new(move || {
            let cors = if cors_enabled {
                Cors::permissive()
            } else {
                Cors::default()
            };

            App::new()
                .app_data(state_data.clone())
                .wrap(cors)
                .wrap(Logger::default())
                .wrap(DefaultHeaders::new().add(("Server", "LiteLLM-Google-Gateway")))
                .route("/health", web::get().to(health_check))
                .route("/v1/models", web::get().to(list_models))
                .route("/v1/chat/completions", web::post().to(chat_completions))
        })
        .bind(&bind_addr)?;

        info!("🚀 Configuration-driven LiteLLM Gateway started successfully!");
        info!("🌐 Listening on: {}", bind_addr);
        info!("📋 API endpoints:");
        info!("   GET  /health - Health check");
        info!("   GET  /v1/models - Model list");
        info!("   POST /v1/chat/completions - Chat completion (actual Google API)");
        info!(
            "🔑 usageGoogle API Key: {}...{}",
            &self.config.google.api_key[..10],
            &self.config.google.api_key[self.config.google.api_key.len() - 4..]
        );
        info!(
            "📊 Enabled models: {}",
            self.config
                .google
                .models
                .iter()
                .filter(|m| m.enabled)
                .map(|m| m.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );

        server.run().await?;
        Ok(())
    }
}

/// Configuration
fn load_config(
    config_path: &str,
) -> Result<GatewayConfig, Box<dyn std::error::Error + Send + Sync>> {
    let config_content = std::fs::read_to_string(config_path)
        .map_err(|e| format!("Unable to read config file {}: {}", config_path, e))?;

    let config: GatewayConfig = serde_yaml::from_str(&config_content)
        .map_err(|e| format!("Config file format error: {}", e))?;

    Ok(config)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Configuration
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "gateway_config.yaml".to_string());

    // Configuration
    let config = load_config(&config_path)?;

    // Initialize
    let log_level = match config.logging.level.as_str() {
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(true)
        .init();

    info!("🚀 Starting configuration-driven LiteLLM Gateway");
    info!("📄 Config file: {}", config_path);

    // Validation
    if config.google.api_key.is_empty() || config.google.api_key == "your-api-key-here" {
        error!("❌ Please set a valid Google API key in the config file");
        return Err("Missing Google API key".into());
    }

    // Validation
    let enabled_models: Vec<_> = config.google.models.iter().filter(|m| m.enabled).collect();

    if enabled_models.is_empty() {
        error!("❌ No enabled models, please enable at least one model in the config file");
        return Err("No enabled models".into());
    }

    info!("✅ Configuration validation passed");
    info!(
        "📊 Enabled models: {}",
        enabled_models
            .iter()
            .map(|m| m.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Create
    let gateway = ConfigurableGateway::new(config).await?;
    gateway.run().await
}
