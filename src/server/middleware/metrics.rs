//! Metrics middleware for request monitoring

use crate::server::state::AppState;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::web;
use futures::future::{Ready, ready};
use std::future::Future;
use std::pin::Pin;
use std::time::Instant;
use tracing::info;

/// Metrics middleware for Actix-web
pub struct MetricsMiddleware;

impl<S, B> Transform<S, ServiceRequest> for MetricsMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = MetricsMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MetricsMiddlewareService { service }))
    }
}

/// Service implementation for metrics middleware
pub struct MetricsMiddlewareService<S> {
    service: S,
}

/// Request metrics data
#[derive(Clone)]
pub struct MiddlewareRequestMetrics {
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub response_time_ms: u64,
    pub request_size: usize,
    pub response_size: usize,
    pub user_agent: Option<String>,
    pub client_ip: Option<String>,
    pub user_id: Option<String>,
    pub api_key_id: Option<String>,
}

impl<S, B> Service<ServiceRequest> for MetricsMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start_time = Instant::now();
        let method = req.method().to_string();
        let path = req.path().to_string();

        let user_agent = req
            .headers()
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        let client_ip = req.connection_info().peer_addr().map(|s| s.to_string());

        let request_size = req
            .headers()
            .get("content-length")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let app_state = req.app_data::<web::Data<AppState>>().cloned();
        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            let response_time = start_time.elapsed();
            let status_code = res.status().as_u16();

            // Metrics recording is handled by MonitoringSystem if configured
            // For now, just log the request completion
            let _ = (app_state, request_size, user_agent, client_ip);

            info!(
                "{} {} -> {} in {:?}",
                method, path, status_code, response_time
            );

            Ok(res)
        })
    }
}
