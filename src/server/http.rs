//! HTTP server core implementation
//!
//! This module provides the HttpServer struct and its core methods.

use crate::config::{Config, ServerConfig};
use crate::server::handlers::health_check;
use crate::server::middleware::{AuthMiddleware, RequestIdMiddleware};
use crate::server::routes;
use crate::server::state::AppState;
use crate::services::pricing::PricingService;
use crate::utils::error::{GatewayError, Result};
use actix_cors::Cors;
use actix_web::{
    App, HttpServer as ActixHttpServer,
    middleware::{DefaultHeaders, Logger},
    web,
};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// HTTP server
pub struct HttpServer {
    /// Server configuration
    config: ServerConfig,
    /// Application state
    state: AppState,
}

impl HttpServer {
    /// Create a new HTTP server
    pub async fn new(config: &Config) -> Result<Self> {
        info!("Creating HTTP server");

        let storage = crate::storage::StorageLayer::new(&config.gateway.storage).await?;
        let auth =
            crate::auth::AuthSystem::new(&config.gateway.auth, Arc::new(storage.clone())).await?;
        let mut router = crate::core::providers::ProviderRegistry::new();

        // Initialize providers from config
        if !config.gateway.providers.is_empty() {
            for provider_config in &config.gateway.providers {
                let provider_type: crate::core::providers::ProviderType =
                    provider_config.provider_type.as_str().into();

                let mut settings = provider_config.settings.clone();
                // Add api_key from config if not in settings
                if !settings.contains_key("api_key") && !provider_config.api_key.is_empty() {
                    settings.insert(
                        "api_key".to_string(),
                        serde_json::Value::String(provider_config.api_key.clone()),
                    );
                }

                match crate::core::providers::Provider::from_config_async(
                    provider_type.clone(),
                    serde_json::Value::Object(settings.into_iter().collect()),
                )
                .await
                {
                    Ok(provider) => {
                        router.register(provider);
                        info!("Registered provider: {}", provider_config.name);
                    }
                    Err(e) => {
                        warn!(
                            "Failed to initialize provider {}: {}",
                            provider_config.name, e
                        );
                    }
                }
            }
        } else {
            debug!("No providers configured, gateway will route based on model prefix");
        }

        let pricing = Arc::new(PricingService::new(Some(
            "config/model_prices_extended.json".to_string(),
        )));
        let _ = pricing.initialize().await;
        let pricing_clone: Arc<PricingService> = Arc::clone(&pricing);
        let _pricing_task = pricing_clone.start_auto_refresh_task();

        let state = AppState::new(config.clone(), auth, router, storage, pricing);

        Ok(Self {
            config: config.gateway.server.clone(),
            state,
        })
    }

    /// Create the Actix-web application
    fn create_app(
        state: web::Data<AppState>,
    ) -> App<
        impl actix_web::dev::ServiceFactory<
            actix_web::dev::ServiceRequest,
            Config = (),
            Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
            Error = actix_web::Error,
            InitError = (),
        >,
    > {
        info!("Setting up routes and middleware");

        let cors_config = &state.config.gateway.server.cors;
        let mut cors = Cors::default();

        if cors_config.enabled {
            if cors_config.allows_all_origins() {
                cors = cors.allow_any_origin();
                cors_config.validate().unwrap_or_else(|e| {
                    warn!(error = %e, "CORS Configuration Warning");
                });
            } else {
                for origin in &cors_config.allowed_origins {
                    cors = cors.allowed_origin(origin);
                }
            }

            let methods: Vec<actix_web::http::Method> = cors_config
                .allowed_methods
                .iter()
                .filter_map(|m| m.parse().ok())
                .collect();
            if !methods.is_empty() {
                cors = cors.allowed_methods(methods);
            }

            let headers: Vec<actix_web::http::header::HeaderName> = cors_config
                .allowed_headers
                .iter()
                .filter_map(|h| h.parse().ok())
                .collect();
            if !headers.is_empty() {
                cors = cors.allowed_headers(headers);
            }

            cors = cors.max_age(cors_config.max_age as usize);

            if cors_config.allow_credentials {
                cors = cors.supports_credentials();
            }
        }

        App::new()
            .app_data(state)
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(DefaultHeaders::new().add(("Server", "LiteLLM-RS")))
            .wrap(AuthMiddleware)
            .wrap(RequestIdMiddleware)
            .route("/health", web::get().to(health_check))
            .configure(routes::ai::configure_routes)
            .configure(routes::pricing::configure_pricing_routes)
    }

    /// Start the HTTP server
    pub async fn start(self) -> Result<()> {
        let bind_addr = format!("{}:{}", self.config.host, self.config.port);
        let port = self.config.port;

        info!("Starting HTTP server on {}", bind_addr);

        let state = web::Data::new(self.state);

        let server = ActixHttpServer::new(move || Self::create_app(state.clone()))
            .bind(&bind_addr)
            .map_err(|e| Self::format_bind_error(e, &bind_addr, port))?
            .run();

        info!("HTTP server listening on {}", bind_addr);

        server
            .await
            .map_err(|e| GatewayError::server(format!("Server error: {}", e)))?;

        info!("HTTP server stopped");
        Ok(())
    }

    /// Get server configuration
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// Get application state
    pub fn state(&self) -> &AppState {
        &self.state
    }
}
