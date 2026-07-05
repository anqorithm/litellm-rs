//! Request ID middleware

use crate::utils::error::gateway_error::{GatewayError, with_gateway_error_request_id};
use actix_web::body::{BoxBody, MessageBody};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::http::header::HeaderValue;
use futures::future::{Ready, ready};
use std::future::Future;
use std::pin::Pin;
use tracing::debug;
use uuid::Uuid;

/// Request ID middleware for Actix-web
pub struct RequestIdMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RequestIdMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = RequestIdMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestIdMiddlewareService { service }))
    }
}

/// Service implementation for request ID middleware
pub struct RequestIdMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestIdMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let existing = req
            .headers()
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
            .filter(|s| !s.is_empty())
            .map(str::to_string);

        let request_id = if let Some(id) = existing {
            id
        } else {
            let id = Uuid::new_v4().to_string();
            req.headers_mut().insert(
                actix_web::http::header::HeaderName::from_static("x-request-id"),
                HeaderValue::from_str(&id).unwrap_or_else(|_| HeaderValue::from_static("invalid")),
            );
            id
        };

        debug!("Processing request: {}", request_id);

        let fut = self.service.call(req);
        Box::pin(async move {
            let header_name = actix_web::http::header::HeaderName::from_static("x-request-id");
            let header_value = HeaderValue::from_str(&request_id)
                .unwrap_or_else(|_| HeaderValue::from_static("invalid"));

            let result = with_gateway_error_request_id(request_id.clone(), fut).await;
            match result {
                Ok(mut res) => {
                    res.headers_mut().insert(header_name, header_value);
                    Ok(res.map_into_boxed_body())
                }
                Err(err) => {
                    let mut response = if let Some(gateway_error) = err.as_error::<GatewayError>() {
                        gateway_error.error_response_with_request_id(Some(request_id.clone()))
                    } else {
                        err.error_response()
                    };
                    response.headers_mut().insert(header_name, header_value);
                    Err(actix_web::error::InternalError::from_response(err, response).into())
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, HttpResponse, body::to_bytes, http::StatusCode, test, web};
    use serde_json::Value;

    #[actix_web::test]
    async fn middleware_does_not_clone_request_before_routing() {
        let app = test::init_service(App::new().wrap(RequestIdMiddleware).route(
            "/health",
            web::get().to(|| async { HttpResponse::Ok().finish() }),
        ))
        .await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), StatusCode::OK);
        assert!(res.headers().contains_key("x-request-id"));
    }

    #[actix_web::test]
    async fn middleware_adds_request_id_to_error_responses() {
        let app = test::init_service(App::new().wrap(RequestIdMiddleware).route(
            "/error",
            web::get().to(|| async {
                Err::<HttpResponse, _>(actix_web::error::ErrorInternalServerError("test error"))
            }),
        ))
        .await;

        let req = test::TestRequest::get().uri("/error").to_request();
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(res.headers().contains_key("x-request-id"));
    }

    #[actix_web::test]
    async fn middleware_adds_request_id_to_gateway_error_body() {
        let app = test::init_service(App::new().wrap(RequestIdMiddleware).route(
            "/gateway-error",
            web::get().to(|| async {
                Err::<HttpResponse, GatewayError>(GatewayError::Auth("bad token".to_string()))
            }),
        ))
        .await;

        let req = test::TestRequest::get()
            .uri("/gateway-error")
            .insert_header(("x-request-id", "req-test-123"))
            .to_request();
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            res.headers()
                .get("x-request-id")
                .and_then(|value| value.to_str().ok()),
            Some("req-test-123")
        );
        let body = to_bytes(res.into_body()).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["request_id"], "req-test-123");
    }
}
