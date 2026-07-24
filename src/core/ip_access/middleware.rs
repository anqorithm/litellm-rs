//! IP Access Control middleware for Actix-web
//!
//! This module provides middleware for IP-based request filtering.

use actix_web::body::MessageBody;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::http::StatusCode;
use actix_web::{Error, HttpResponse, body::BoxBody};
use futures::future::{LocalBoxFuture, Ready, ready};
use std::{rc::Rc, sync::Arc};
use tracing::{debug, warn};

use super::control::IpAccessControl;

/// IP Access Control middleware for Actix-web
pub struct IpAccessMiddleware {
    controller: Arc<IpAccessControl>,
}

impl IpAccessMiddleware {
    /// Create a new IP access middleware
    pub fn new(controller: Arc<IpAccessControl>) -> Self {
        Self { controller }
    }
}

impl<S, B> Transform<S, ServiceRequest> for IpAccessMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = IpAccessMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(IpAccessMiddlewareService {
            service: Rc::new(service),
            controller: self.controller.clone(),
        }))
    }
}

/// Service implementation for IP access middleware
pub struct IpAccessMiddlewareService<S> {
    service: Rc<S>,
    controller: Arc<IpAccessControl>,
}

impl<S, B> Service<ServiceRequest> for IpAccessMiddlewareService<S>
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
        let controller = self.controller.clone();
        let path = req.path().to_string();

        // Check if path is excluded
        if controller.is_path_excluded(&path) {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_boxed_body())
            });
        }

        // Check if IP access control is enabled
        if !controller.is_enabled() {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_boxed_body())
            });
        }

        // Extract client IP
        let remote_addr = req
            .connection_info()
            .peer_addr()
            .unwrap_or("unknown")
            .to_string();

        let forwarded_for = req
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let client_ip = controller.extract_client_ip(&remote_addr, forwarded_for.as_deref());

        let config = controller.config().clone();
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            // Check if IP is allowed
            if !controller.is_allowed(&client_ip).await {
                if config.log_blocked {
                    warn!("IP access denied for: {}", client_ip);
                }

                let response = HttpResponse::build(
                    StatusCode::from_u16(config.blocked_status).unwrap_or(StatusCode::FORBIDDEN),
                )
                .content_type("application/json")
                .body(
                    serde_json::json!({
                        "error": {
                            "message": config.blocked_message,
                            "type": "ip_access_denied",
                            "code": "forbidden"
                        }
                    })
                    .to_string(),
                );

                return Ok(req.into_response(response).map_into_boxed_body());
            }

            debug!("IP access granted for: {}", client_ip);
            let res = service.call(req).await?;
            Ok(res.map_into_boxed_body())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ip_access::config::{IpAccessConfig, IpRuleConfig};
    use crate::core::ip_access::types::IpAccessMode;
    use actix_web::{App, test as actix_test, web};
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_middleware_creation() {
        let config = IpAccessConfig::default();
        let controller = Arc::new(IpAccessControl::new(config).unwrap());
        let _middleware = IpAccessMiddleware::new(controller);
    }

    #[actix_web::test]
    async fn denied_request_never_executes_downstream_service() {
        let config = IpAccessConfig {
            enabled: true,
            mode: IpAccessMode::Blocklist,
            blocklist: vec![IpRuleConfig::new("127.0.0.1")],
            ..IpAccessConfig::default()
        };
        let controller = Arc::new(IpAccessControl::new(config).expect("valid IP policy"));
        let calls = Arc::new(AtomicUsize::new(0));
        let app = actix_test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::clone(&calls)))
                .wrap(IpAccessMiddleware::new(controller))
                .route(
                    "/sentinel",
                    web::get().to(|calls: web::Data<Arc<AtomicUsize>>| async move {
                        calls.fetch_add(1, Ordering::SeqCst);
                        HttpResponse::Ok().finish()
                    }),
                ),
        )
        .await;
        let request = actix_test::TestRequest::get()
            .uri("/sentinel")
            .peer_addr("127.0.0.1:9000".parse().expect("valid socket address"))
            .to_request();

        let response = actix_test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[actix_web::test]
    async fn untrusted_forwarded_for_cannot_spoof_the_client_ip() {
        let config = IpAccessConfig {
            enabled: true,
            mode: IpAccessMode::Blocklist,
            blocklist: vec![IpRuleConfig::new("203.0.113.9")],
            trust_proxy: false,
            ..IpAccessConfig::default()
        };
        let controller = Arc::new(IpAccessControl::new(config).expect("valid IP policy"));
        let app =
            actix_test::init_service(App::new().wrap(IpAccessMiddleware::new(controller)).route(
                "/sentinel",
                web::get().to(|| async { HttpResponse::Ok().finish() }),
            ))
            .await;
        let request = actix_test::TestRequest::get()
            .uri("/sentinel")
            .insert_header(("x-forwarded-for", "203.0.113.9"))
            .peer_addr("127.0.0.1:9000".parse().expect("valid socket address"))
            .to_request();

        let response = actix_test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::OK);
    }
}
