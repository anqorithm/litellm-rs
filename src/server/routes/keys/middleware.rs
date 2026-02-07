//! Middleware for API key management routes
//!
//! This module provides middleware for authentication, rate limiting,
//! and audit logging of key management operations.

#![allow(dead_code)] // Middleware structures are defined for future use

use actix_web::Error;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use futures::future::LocalBoxFuture;
use std::future::{Ready, ready};
use std::rc::Rc;
use tracing::{info, warn};

/// Middleware for audit logging key operations
pub struct KeyAuditLogger;

impl KeyAuditLogger {
    pub fn new() -> Self {
        Self
    }
}

impl Default for KeyAuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, B> Transform<S, ServiceRequest> for KeyAuditLogger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = KeyAuditLoggerMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(KeyAuditLoggerMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct KeyAuditLoggerMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for KeyAuditLoggerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        // Extract audit information
        let method = req.method().to_string();
        let path = req.path().to_string();
        let client_ip = req
            .connection_info()
            .peer_addr()
            .unwrap_or("unknown")
            .to_string();

        Box::pin(async move {
            // Log the request
            info!(
                target: "key_audit",
                method = %method,
                path = %path,
                client_ip = %client_ip,
                "Key management operation initiated"
            );

            let result = service.call(req).await;

            match &result {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        info!(
                            target: "key_audit",
                            method = %method,
                            path = %path,
                            status = %status.as_u16(),
                            "Key management operation completed successfully"
                        );
                    } else {
                        warn!(
                            target: "key_audit",
                            method = %method,
                            path = %path,
                            status = %status.as_u16(),
                            "Key management operation failed"
                        );
                    }
                }
                Err(e) => {
                    warn!(
                        target: "key_audit",
                        method = %method,
                        path = %path,
                        error = %e,
                        "Key management operation error"
                    );
                }
            }

            result
        })
    }
}

/// Rate limiter for key verification endpoint
/// This helps prevent brute-force attacks on the verification endpoint
pub struct KeyVerificationRateLimiter {
    /// Maximum requests per minute
    pub max_requests_per_minute: u32,
}

impl KeyVerificationRateLimiter {
    pub fn new(max_requests_per_minute: u32) -> Self {
        Self {
            max_requests_per_minute,
        }
    }
}

impl Default for KeyVerificationRateLimiter {
    fn default() -> Self {
        Self::new(60) // 60 requests per minute by default
    }
}

#[cfg(test)]
mod middleware_tests {
    use super::*;

    #[test]
    fn test_audit_logger_creation() {
        let _logger = KeyAuditLogger::new();
    }

    #[test]
    fn test_rate_limiter_default() {
        let limiter = KeyVerificationRateLimiter::default();
        assert_eq!(limiter.max_requests_per_minute, 60);
    }

    #[test]
    fn test_rate_limiter_custom() {
        let limiter = KeyVerificationRateLimiter::new(100);
        assert_eq!(limiter.max_requests_per_minute, 100);
    }
}
