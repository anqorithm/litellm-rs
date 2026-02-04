//! Current user endpoint and helpers

use crate::core::models::user::types::User;
use crate::server::routes::ApiResponse;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Result as ActixResult};
use tracing::debug;

/// Get current user endpoint
pub async fn get_current_user(req: HttpRequest) -> ActixResult<HttpResponse> {
    debug!("Get current user request");

    // Get authenticated user
    let user = match get_authenticated_user(&req) {
        Some(user) => user,
        None => {
            return Ok(HttpResponse::Unauthorized()
                .json(ApiResponse::<()>::error("Unauthorized".to_string())));
        }
    };

    Ok(HttpResponse::Ok().json(ApiResponse::success(user)))
}

/// Get authenticated user from request extensions
pub fn get_authenticated_user(req: &HttpRequest) -> Option<User> {
    req.extensions().get::<User>().cloned()
}
