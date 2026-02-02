//! Audit middleware for Actix-web
//!
//! This module provides middleware for automatic request/response logging.

use actix_web::body::MessageBody;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::Error;
use futures::future::{Ready, ready, LocalBoxFuture};
use std::sync::Arc;
use std::time::Instant;
use tracing::debug;

use super::events::AuditEvent;
use super::logger::AuditLogger;
use super::types::RequestLog;

/// Audit middleware for Actix-web
pub struct AuditMiddleware {
    logger: Arc<AuditLogger>,
}

impl AuditMiddleware {
    /// Create a new audit middleware
    pub fn new(logger: Arc<AuditLogger>) -> Self {
        Self { logger }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuditMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuditMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuditMiddlewareService {
            service,
            logger: self.logger.clone(),
        }))
    }
}

/// Service implementation for audit middleware
pub struct AuditMiddlewareService<S> {
    service: S,
    logger: Arc<AuditLogger>,
}

impl<S, B> Service<ServiceRequest> for AuditMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let logger = self.logger.clone();
        let path = req.path().to_string();
        let method = req.method().to_string();

        // Check if path should be logged
        if !logger.should_log_path(&path) {
            let fut = self.service.call(req);
            return Box::pin(fut);
        }

        // Generate request ID
        let request_id = req
            .headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        // Extract request info
        let client_ip = req
            .connection_info()
            .realip_remote_addr()
            .map(|s| s.to_string());

        let user_agent = req
            .headers()
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Create request log
        let mut request_log = RequestLog::new(&request_id, &method, &path);
        if let Some(ip) = client_ip {
            request_log = request_log.with_client_ip(ip);
        }
        if let Some(ua) = user_agent {
            request_log = request_log.with_user_agent(ua);
        }

        // Log request started
        let start_event = AuditEvent::request_started(&request_id, &path)
            .with_request(request_log);

        logger.log_sync(start_event);

        let start_time = Instant::now();
        let fut = self.service.call(req);

        Box::pin(async move {
            let result = fut.await;
            let duration_ms = start_time.elapsed().as_millis() as u64;

            match &result {
                Ok(response) => {
                    let status_code = response.status().as_u16();
                    let event = AuditEvent::request_completed(&request_id, status_code, duration_ms);
                    logger.log(event).await;
                    debug!(
                        "Request {} completed: status={}, duration={}ms",
                        request_id, status_code, duration_ms
                    );
                }
                Err(e) => {
                    let event = AuditEvent::request_failed(&request_id, e.to_string());
                    logger.log(event).await;
                    debug!("Request {} failed: {}", request_id, e);
                }
            }

            result
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_middleware_creation() {
        let logger = Arc::new(AuditLogger::disabled());
        let _middleware = AuditMiddleware::new(logger);
    }
}
