//! Guardrails middleware for Actix-web
//!
//! This module provides middleware for integrating guardrails into the HTTP pipeline.

use actix_web::body::MessageBody;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::http::StatusCode;
use actix_web::{Error, HttpResponse, body::BoxBody};
use futures::future::{Ready, ready, LocalBoxFuture};
use std::sync::Arc;
use tracing::debug;

use super::engine::GuardrailEngine;
use super::types::GuardrailAction;

/// Guardrails middleware for Actix-web
pub struct GuardrailMiddleware {
    engine: Arc<GuardrailEngine>,
}

impl GuardrailMiddleware {
    /// Create a new guardrails middleware
    pub fn new(engine: Arc<GuardrailEngine>) -> Self {
        Self { engine }
    }
}

impl<S, B> Transform<S, ServiceRequest> for GuardrailMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = GuardrailMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(GuardrailMiddlewareService {
            service,
            engine: self.engine.clone(),
        }))
    }
}

/// Service implementation for guardrails middleware
pub struct GuardrailMiddlewareService<S> {
    service: S,
    engine: Arc<GuardrailEngine>,
}

impl<S, B> Service<ServiceRequest> for GuardrailMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let engine = self.engine.clone();
        let path = req.path().to_string();

        // Check if path is excluded
        if engine.is_path_excluded(&path) {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_boxed_body())
            });
        }

        // For now, we pass through - full implementation would extract body
        // and check it against guardrails
        let fut = self.service.call(req);

        Box::pin(async move {
            debug!("Guardrails middleware processing request to {}", path);
            let res = fut.await?;
            Ok(res.map_into_boxed_body())
        })
    }
}

/// Create a blocked response
#[allow(dead_code)]
fn create_blocked_response(message: &str) -> HttpResponse<BoxBody> {
    HttpResponse::build(StatusCode::BAD_REQUEST)
        .content_type("application/json")
        .body(serde_json::json!({
            "error": {
                "message": message,
                "type": "guardrail_violation",
                "code": "content_blocked"
            }
        }).to_string())
}

/// Context stored in request extensions after guardrail check
#[derive(Debug, Clone)]
pub struct GuardrailCheckContext {
    /// Whether the check passed
    pub passed: bool,
    /// Action taken
    pub action: GuardrailAction,
    /// Number of violations
    pub violation_count: usize,
    /// Modified content (if masking was applied)
    pub modified_content: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::guardrails::config::GuardrailConfig;

    #[test]
    fn test_middleware_creation() {
        let config = GuardrailConfig::default();
        let engine = Arc::new(GuardrailEngine::new(config).unwrap());
        let _middleware = GuardrailMiddleware::new(engine);
    }

    #[test]
    fn test_blocked_response() {
        let response = create_blocked_response("Content blocked");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_guardrail_check_context() {
        let context = GuardrailCheckContext {
            passed: true,
            action: GuardrailAction::Allow,
            violation_count: 0,
            modified_content: None,
        };
        assert!(context.passed);
        assert_eq!(context.violation_count, 0);
    }
}
