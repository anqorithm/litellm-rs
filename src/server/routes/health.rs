//! Health check and status endpoints
//!
//! This module provides health check and system status endpoints.

#![allow(dead_code)]

use crate::server::routes::ApiResponse;
use crate::server::state::AppState;
use actix_web::{HttpResponse, Result as ActixResult, web};
use std::borrow::Cow;

use tracing::{debug, error};

/// Configure health check routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/health")
            .route("", web::get().to(health_check))
            .route("/detailed", web::get().to(detailed_health_check)),
    )
    .route("/status", web::get().to(system_status))
    .route("/version", web::get().to(version_info))
    .route("/metrics", web::get().to(metrics));
}

/// Basic health check endpoint
///
/// Returns a simple health status indicating if the service is running.
/// This endpoint is typically used by load balancers and monitoring systems.
pub async fn health_check(_state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    debug!("Health check requested");

    let health_status = HealthStatus {
        status: Cow::Borrowed("healthy"),
        timestamp: chrono::Utc::now(),
        version: Cow::Borrowed(env!("CARGO_PKG_VERSION")),
    };

    Ok(HttpResponse::Ok().json(ApiResponse::success(health_status)))
}

/// Detailed health check endpoint
///
/// Returns comprehensive health information including storage, authentication,
/// and provider status. This endpoint provides more detailed diagnostics.
async fn detailed_health_check(state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    debug!("Detailed health check requested");

    // Check storage health
    let storage_health = if state.config.storage().database.url.is_empty() {
        crate::storage::StorageHealthStatus {
            overall: false,
            database: false,
            redis: false,
            files: false,
            vector: false,
        }
    } else {
        // Get actual storage health
        match state.storage.health_check().await {
            Ok(status) => status,
            Err(_) => crate::storage::StorageHealthStatus {
                overall: false,
                database: false,
                redis: false,
                files: false,
                vector: false,
            },
        }
    };

    // Check provider health
    let provider_health = match check_provider_health(&state).await {
        Ok(status) => status,
        Err(e) => {
            error!("Provider health check failed: {}", e);
            ProviderHealthStatus {
                healthy_providers: 0,
                total_providers: 0,
                provider_details: vec![],
            }
        }
    };

    let detailed_status = DetailedHealthStatus {
        status: if storage_health.overall && provider_health.healthy_providers > 0 {
            Cow::Borrowed("healthy")
        } else {
            Cow::Borrowed("degraded")
        },
        timestamp: chrono::Utc::now(),
        version: Cow::Borrowed(env!("CARGO_PKG_VERSION")),
        uptime_seconds: get_uptime_seconds(),
        storage: storage_health,
        providers: provider_health,
        memory_usage: get_memory_usage(),
        cpu_usage: get_cpu_usage(),
    };

    Ok(HttpResponse::Ok().json(ApiResponse::success(detailed_status)))
}

/// System status endpoint
///
/// Returns general system information and statistics.
async fn system_status(state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    debug!("System status requested");

    let system_status = SystemStatus {
        service_name: Cow::Borrowed("Rust LiteLLM Gateway"),
        version: Cow::Borrowed(env!("CARGO_PKG_VERSION")),
        build_time: Cow::Borrowed(env!("BUILD_TIME")),
        git_hash: Cow::Borrowed(env!("GIT_HASH")),
        rust_version: Cow::Borrowed(env!("RUST_VERSION")),
        uptime_seconds: get_uptime_seconds(),
        timestamp: chrono::Utc::now(),
        environment: std::env::var("ENVIRONMENT")
            .map(Cow::Owned)
            .unwrap_or(Cow::Borrowed("development")),
        config: SystemConfig {
            server_host: state.config.server().host.clone(),
            server_port: state.config.server().port,
            auth_enabled: state.config.auth().enable_jwt || state.config.auth().enable_api_key,
            rate_limiting_enabled: state.config.gateway.rate_limit.enabled,
            caching_enabled: state.config.gateway.cache.enabled,
            providers_count: state.config.providers().len(),
        },
    };

    Ok(HttpResponse::Ok().json(ApiResponse::success(system_status)))
}

/// Version information endpoint
///
/// Returns version and build information.
async fn version_info() -> HttpResponse {
    debug!("Version info requested");

    let version_info = VersionInfo {
        version: Cow::Borrowed(env!("CARGO_PKG_VERSION")),
        build_time: Cow::Borrowed(env!("BUILD_TIME")),
        git_hash: Cow::Borrowed(env!("GIT_HASH")),
        rust_version: Cow::Borrowed(env!("RUST_VERSION")),
        features: get_enabled_features(),
    };

    HttpResponse::Ok().json(ApiResponse::success(version_info))
}

/// Metrics endpoint (Prometheus format)
///
/// Returns metrics in Prometheus format for monitoring systems.
async fn metrics(state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    debug!("Metrics requested");

    // TODO: Implement proper Prometheus metrics
    // For now, return basic metrics in Prometheus format
    let metrics = format!(
        r#"# HELP gateway_uptime_seconds Total uptime of the gateway in seconds
# TYPE gateway_uptime_seconds counter
gateway_uptime_seconds {}

# HELP gateway_memory_usage_bytes Current memory usage in bytes
# TYPE gateway_memory_usage_bytes gauge
gateway_memory_usage_bytes {}

# HELP gateway_cpu_usage_percent Current CPU usage percentage
# TYPE gateway_cpu_usage_percent gauge
gateway_cpu_usage_percent {}

# HELP gateway_providers_total Total number of configured providers
# TYPE gateway_providers_total gauge
gateway_providers_total {}
"#,
        get_uptime_seconds(),
        get_memory_usage(),
        get_cpu_usage(),
        state.config.providers().len()
    );

    Ok(HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(metrics))
}

/// Basic health status
#[derive(Debug, Clone, serde::Serialize)]
struct HealthStatus {
    status: Cow<'static, str>,
    timestamp: chrono::DateTime<chrono::Utc>,
    version: Cow<'static, str>,
}

/// Detailed health status
#[derive(Debug, Clone, serde::Serialize)]
struct DetailedHealthStatus {
    status: Cow<'static, str>,
    timestamp: chrono::DateTime<chrono::Utc>,
    version: Cow<'static, str>,
    uptime_seconds: u64,
    storage: crate::storage::StorageHealthStatus,
    providers: ProviderHealthStatus,
    memory_usage: u64,
    cpu_usage: f64,
}

/// Provider health status
#[derive(Debug, Clone, serde::Serialize)]
struct ProviderHealthStatus {
    healthy_providers: usize,
    total_providers: usize,
    provider_details: Vec<ProviderHealth>,
}

/// Individual provider health
#[derive(Debug, Clone, serde::Serialize)]
struct ProviderHealth {
    name: String,
    status: Cow<'static, str>,
    response_time_ms: Option<u64>,
    last_check: chrono::DateTime<chrono::Utc>,
    error_message: Option<String>,
}

/// System status information
#[derive(Debug, Clone, serde::Serialize)]
struct SystemStatus {
    service_name: Cow<'static, str>,
    version: Cow<'static, str>,
    build_time: Cow<'static, str>,
    git_hash: Cow<'static, str>,
    rust_version: Cow<'static, str>,
    uptime_seconds: u64,
    timestamp: chrono::DateTime<chrono::Utc>,
    environment: Cow<'static, str>,
    config: SystemConfig,
}

/// System configuration summary
#[derive(Debug, Clone, serde::Serialize)]
struct SystemConfig {
    server_host: String,
    server_port: u16,
    auth_enabled: bool,
    rate_limiting_enabled: bool,
    caching_enabled: bool,
    providers_count: usize,
}

/// Version information
#[derive(Debug, Clone, serde::Serialize)]
struct VersionInfo {
    version: Cow<'static, str>,
    build_time: Cow<'static, str>,
    git_hash: Cow<'static, str>,
    rust_version: Cow<'static, str>,
    features: Vec<Cow<'static, str>>,
}

/// Check provider health
async fn check_provider_health(
    state: &AppState,
) -> Result<ProviderHealthStatus, crate::utils::error::gateway_error::GatewayError> {
    let mut provider_details = Vec::new();
    let mut healthy_count = 0;

    for provider_config in state.config.providers() {
        let start_time = std::time::Instant::now();

        // TODO: Implement actual provider health checks
        // For now, assume all providers are healthy
        let status: Cow<'static, str> = Cow::Borrowed("healthy");
        let response_time = start_time.elapsed().as_millis() as u64;

        if status == "healthy" {
            healthy_count += 1;
        }

        provider_details.push(ProviderHealth {
            name: provider_config.name.clone(),
            status,
            response_time_ms: Some(response_time),
            last_check: chrono::Utc::now(),
            error_message: None,
        });
    }

    Ok(ProviderHealthStatus {
        healthy_providers: healthy_count,
        total_providers: state.config.providers().len(),
        provider_details,
    })
}

/// Get system uptime in seconds
fn get_uptime_seconds() -> u64 {
    // This is a simplified implementation
    // In a real application, you would track the actual start time
    static START_TIME: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    let start = START_TIME.get_or_init(std::time::Instant::now);
    start.elapsed().as_secs()
}

/// Get memory usage in bytes
fn get_memory_usage() -> u64 {
    // This is a placeholder implementation
    // In a real application, you would use a proper memory monitoring library
    0
}

/// Get CPU usage percentage
fn get_cpu_usage() -> f64 {
    // This is a placeholder implementation
    // In a real application, you would use a proper CPU monitoring library
    0.0
}

/// Get enabled features
fn get_enabled_features() -> Vec<Cow<'static, str>> {
    let mut features = Vec::new();

    #[cfg(feature = "enterprise")]
    features.push(Cow::Borrowed("enterprise"));

    #[cfg(feature = "analytics")]
    features.push(Cow::Borrowed("analytics"));

    #[cfg(feature = "vector-db")]
    features.push(Cow::Borrowed("vector-db"));

    if features.is_empty() {
        features.push(Cow::Borrowed("standard"));
    }

    features
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_creation() {
        let status = HealthStatus {
            status: Cow::Borrowed("healthy"),
            timestamp: chrono::Utc::now(),
            version: Cow::Borrowed("1.0.0"),
        };

        assert_eq!(status.status, "healthy");
        assert_eq!(status.version, "1.0.0");
    }

    #[test]
    fn test_provider_health_status() {
        let provider_health = ProviderHealthStatus {
            healthy_providers: 2,
            total_providers: 3,
            provider_details: vec![],
        };

        assert_eq!(provider_health.healthy_providers, 2);
        assert_eq!(provider_health.total_providers, 3);
    }

    #[test]
    fn test_version_info() {
        let version_info = VersionInfo {
            version: Cow::Borrowed("1.0.0"),
            build_time: Cow::Borrowed("2024-01-01T00:00:00Z"),
            git_hash: Cow::Borrowed("abc123"),
            rust_version: Cow::Borrowed("1.75.0"),
            features: vec![Cow::Borrowed("standard")],
        };

        assert_eq!(version_info.version, "1.0.0");
        assert!(!version_info.features.is_empty());
    }

    #[test]
    fn test_get_enabled_features() {
        let features = get_enabled_features();
        assert!(!features.is_empty());
        // With --all-features, we may have enterprise/analytics/vector-db instead of standard
        // Just ensure we get some valid features
        let valid_features = ["standard", "enterprise", "analytics", "vector-db"];
        assert!(
            features
                .iter()
                .any(|f| valid_features.contains(&f.as_ref()))
        );
    }
}
