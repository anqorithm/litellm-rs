//! Langfuse Middleware
//!
//! Actix-web middleware for automatic HTTP request tracing with Langfuse.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use actix_web::Error;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::http::header::HeaderValue;
use chrono::Utc;
use futures::future::{Ready, ok};
use tracing::debug;

use super::client::LangfuseClient;
use super::config::LangfuseConfig;
use super::types::{IngestionEvent, Span, Trace};

/// Header names for trace context propagation
pub const TRACE_ID_HEADER: &str = "x-langfuse-trace-id";
pub const PARENT_SPAN_ID_HEADER: &str = "x-langfuse-parent-span-id";
pub const SESSION_ID_HEADER: &str = "x-langfuse-session-id";
pub const USER_ID_HEADER: &str = "x-langfuse-user-id";

/// Extract trace ID from request headers or generate new one
fn extract_or_generate_trace_id(req: &ServiceRequest) -> String {
    req.headers()
        .get(TRACE_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(super::types::generate_id)
}

/// Extract optional header value
fn extract_header(req: &ServiceRequest, name: &str) -> Option<String> {
    req.headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Langfuse tracing middleware
///
/// Automatically creates traces and spans for HTTP requests.
pub struct LangfuseTracing {
    client: Option<Arc<LangfuseClient>>,
    /// Include request body in traces
    include_request_body: bool,
    /// Include response body in traces
    include_response_body: bool,
    /// Paths to exclude from tracing
    exclude_paths: Vec<String>,
    /// Service name for traces
    service_name: String,
}

impl LangfuseTracing {
    /// Create new tracing middleware
    pub fn new(config: LangfuseConfig) -> Self {
        let client = match LangfuseClient::new(config) {
            Ok(c) => Some(Arc::new(c)),
            Err(e) => {
                tracing::warn!("Failed to create Langfuse client: {}", e);
                None
            }
        };

        Self {
            client,
            include_request_body: false,
            include_response_body: false,
            exclude_paths: vec![
                "/health".to_string(),
                "/metrics".to_string(),
                "/ready".to_string(),
                "/live".to_string(),
            ],
            service_name: "litellm-rs".to_string(),
        }
    }

    /// Create middleware from environment variables
    pub fn from_env() -> Self {
        Self::new(LangfuseConfig::from_env())
    }

    /// Set whether to include request body
    pub fn include_request_body(mut self, include: bool) -> Self {
        self.include_request_body = include;
        self
    }

    /// Set whether to include response body
    pub fn include_response_body(mut self, include: bool) -> Self {
        self.include_response_body = include;
        self
    }

    /// Set paths to exclude from tracing
    pub fn exclude_paths(mut self, paths: Vec<String>) -> Self {
        self.exclude_paths = paths;
        self
    }

    /// Add a path to exclude
    pub fn exclude_path(mut self, path: impl Into<String>) -> Self {
        self.exclude_paths.push(path.into());
        self
    }

    /// Set service name
    pub fn service_name(mut self, name: impl Into<String>) -> Self {
        self.service_name = name.into();
        self
    }

    /// Check if path should be traced
    fn should_trace(&self, path: &str) -> bool {
        !self.exclude_paths.iter().any(|p| path.starts_with(p))
    }
}

impl<S, B> Transform<S, ServiceRequest> for LangfuseTracing
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = LangfuseTracingMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(LangfuseTracingMiddleware {
            service,
            client: self.client.clone(),
            include_request_body: self.include_request_body,
            include_response_body: self.include_response_body,
            exclude_paths: self.exclude_paths.clone(),
            service_name: self.service_name.clone(),
        })
    }
}

/// Middleware service implementation
pub struct LangfuseTracingMiddleware<S> {
    service: S,
    client: Option<Arc<LangfuseClient>>,
    include_request_body: bool,
    include_response_body: bool,
    exclude_paths: Vec<String>,
    service_name: String,
}

impl<S> LangfuseTracingMiddleware<S> {
    fn should_trace(&self, path: &str) -> bool {
        !self.exclude_paths.iter().any(|p| path.starts_with(p))
    }
}

impl<S, B> Service<ServiceRequest> for LangfuseTracingMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let path = req.path().to_string();
        let method = req.method().to_string();

        // Skip tracing for excluded paths
        if !self.should_trace(&path) || self.client.is_none() {
            let fut = self.service.call(req);
            return Box::pin(fut);
        }

        let Some(client) = self.client.clone() else {
            let fut = self.service.call(req);
            return Box::pin(fut);
        };
        let start_time = Utc::now();
        let start_instant = std::time::Instant::now();

        // Extract trace context
        let trace_id = extract_or_generate_trace_id(&req);
        let parent_span_id = extract_header(&req, PARENT_SPAN_ID_HEADER);
        let session_id = extract_header(&req, SESSION_ID_HEADER);
        let user_id = extract_header(&req, USER_ID_HEADER);

        // Get request metadata
        let uri = req.uri().to_string();
        let query = req.query_string().to_string();
        let content_type = req
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Add trace ID to request headers for downstream services
        if let Ok(header_value) = HeaderValue::from_str(&trace_id) {
            req.headers_mut().insert(
                actix_web::http::header::HeaderName::from_static(TRACE_ID_HEADER),
                header_value,
            );
        }

        let service_name = self.service_name.clone();
        let _include_response = self.include_response_body;

        // Create span
        let span_id = super::types::generate_id();
        let mut span = Span::new(&trace_id)
            .name(format!("{} {}", method, path))
            .input(serde_json::json!({
                "method": method,
                "path": path,
                "uri": uri,
                "query": if query.is_empty() { None } else { Some(query) },
                "content_type": content_type,
            }));

        span.id = span_id.clone();
        span.start_time = Some(start_time);
        span.parent_observation_id = parent_span_id;

        // Create trace if this is a root span
        let mut trace = Trace::with_id(&trace_id)
            .name(format!("{} {}", method, path))
            .metadata("service", serde_json::json!(service_name))
            .metadata("http.method", serde_json::json!(method))
            .metadata("http.path", serde_json::json!(path));

        if let Some(ref uid) = user_id {
            trace = trace.user_id(uid);
        }
        if let Some(ref sid) = session_id {
            trace = trace.session_id(sid);
        }

        let fut = self.service.call(req);

        Box::pin(async move {
            // Queue trace create event
            let trace_event = IngestionEvent::trace_create(trace);
            let span_event = IngestionEvent::span_create(span.clone());

            let mut batch = super::types::IngestionBatch::new();
            batch.add(trace_event);
            batch.add(span_event);

            // Execute request
            let result = fut.await;

            // Calculate duration
            let duration_ms = start_instant.elapsed().as_millis() as u64;
            let end_time = Utc::now();

            // Update span with response info
            let (status_code, level) = match &result {
                Ok(res) => {
                    let status = res.status().as_u16();
                    let level = if status >= 500 {
                        super::types::Level::Error
                    } else if status >= 400 {
                        super::types::Level::Warning
                    } else {
                        super::types::Level::Default
                    };
                    (status, level)
                }
                Err(_) => (500, super::types::Level::Error),
            };

            let mut completed_span = Span::new(&trace_id)
                .output(serde_json::json!({
                    "status_code": status_code,
                    "duration_ms": duration_ms,
                }))
                .level(level);

            completed_span.id = span_id;
            completed_span.end_time = Some(end_time);

            // Queue span update
            batch.add(IngestionEvent::span_update(completed_span));

            // Send events asynchronously
            let client_clone = client.clone();
            tokio::spawn(async move {
                if let Err(e) = client_clone.ingest(batch).await {
                    tracing::warn!("Failed to send Langfuse events: {}", e);
                }
            });

            debug!(
                "Langfuse: Traced {} {} -> {} ({}ms)",
                method, path, status_code, duration_ms
            );

            result
        })
    }
}

/// Extension trait for extracting trace context from requests
pub trait LangfuseRequestExt {
    /// Get the Langfuse trace ID
    fn trace_id(&self) -> Option<String>;

    /// Get the Langfuse session ID
    fn session_id(&self) -> Option<String>;

    /// Get the Langfuse user ID
    fn user_id(&self) -> Option<String>;
}

impl LangfuseRequestExt for actix_web::HttpRequest {
    fn trace_id(&self) -> Option<String> {
        self.headers()
            .get(TRACE_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }

    fn session_id(&self) -> Option<String> {
        self.headers()
            .get(SESSION_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }

    fn user_id(&self) -> Option<String> {
        self.headers()
            .get(USER_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> LangfuseConfig {
        LangfuseConfig {
            public_key: Some("pk-test".to_string()),
            secret_key: Some("sk-test".to_string()),
            host: "https://cloud.langfuse.com".to_string(),
            enabled: true,
            batch_size: 10,
            flush_interval_ms: 1000,
            debug: true,
            release: None,
        }
    }

    #[test]
    fn test_middleware_creation() {
        let middleware = LangfuseTracing::new(test_config());
        assert!(middleware.client.is_some());
    }

    #[test]
    fn test_middleware_from_env() {
        // Will likely not have valid env vars, so client will be None
        let middleware = LangfuseTracing::from_env();
        // Just ensure it doesn't panic
        let _ = middleware;
    }

    #[test]
    fn test_middleware_builder() {
        let middleware = LangfuseTracing::new(test_config())
            .include_request_body(true)
            .include_response_body(true)
            .exclude_path("/api/internal")
            .service_name("my-service");

        assert!(middleware.include_request_body);
        assert!(middleware.include_response_body);
        assert!(
            middleware
                .exclude_paths
                .contains(&"/api/internal".to_string())
        );
        assert_eq!(middleware.service_name, "my-service");
    }

    #[test]
    fn test_should_trace() {
        let middleware = LangfuseTracing::new(test_config())
            .exclude_paths(vec!["/health".to_string(), "/metrics".to_string()]);

        assert!(!middleware.should_trace("/health"));
        assert!(!middleware.should_trace("/health/live"));
        assert!(!middleware.should_trace("/metrics"));
        assert!(middleware.should_trace("/api/chat"));
        assert!(middleware.should_trace("/v1/completions"));
    }

    #[test]
    fn test_default_exclude_paths() {
        let middleware = LangfuseTracing::new(test_config());

        assert!(!middleware.should_trace("/health"));
        assert!(!middleware.should_trace("/metrics"));
        assert!(!middleware.should_trace("/ready"));
        assert!(!middleware.should_trace("/live"));
    }

    #[test]
    fn test_header_constants() {
        assert_eq!(TRACE_ID_HEADER, "x-langfuse-trace-id");
        assert_eq!(PARENT_SPAN_ID_HEADER, "x-langfuse-parent-span-id");
        assert_eq!(SESSION_ID_HEADER, "x-langfuse-session-id");
        assert_eq!(USER_ID_HEADER, "x-langfuse-user-id");
    }

    #[test]
    fn test_disabled_config() {
        let config = LangfuseConfig {
            enabled: false,
            ..Default::default()
        };
        let middleware = LangfuseTracing::new(config);
        assert!(middleware.client.is_none());
    }

    #[test]
    fn test_missing_credentials() {
        let config = LangfuseConfig::default();
        let middleware = LangfuseTracing::new(config);
        assert!(middleware.client.is_none());
    }
}
