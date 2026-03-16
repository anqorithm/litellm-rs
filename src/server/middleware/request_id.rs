//! Request ID middleware

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

        // Clone the HttpRequest so we can build an error ServiceResponse when the
        // inner service returns Err — ensuring x-request-id appears on all responses.
        let http_req = req.request().clone();
        let fut = self.service.call(req);
        Box::pin(async move {
            let header_name = actix_web::http::header::HeaderName::from_static("x-request-id");
            let header_value = HeaderValue::from_str(&request_id)
                .unwrap_or_else(|_| HeaderValue::from_static("invalid"));

            match fut.await {
                Ok(mut res) => {
                    res.headers_mut().insert(header_name, header_value);
                    Ok(res.map_into_boxed_body())
                }
                Err(err) => {
                    let mut response = err.error_response();
                    response.headers_mut().insert(header_name, header_value);
                    Ok(ServiceResponse::new(http_req, response))
                }
            }
        })
    }
}
