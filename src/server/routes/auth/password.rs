//! Password management endpoints

use crate::server::routes::ApiResponse;
use crate::server::state::AppState;
use crate::utils::data::validation::DataValidator;
use actix_web::{HttpRequest, HttpResponse, Result as ActixResult, web};
use tracing::{info, warn};

use super::models::{ChangePasswordRequest, ForgotPasswordRequest, ResetPasswordRequest};
use super::user::get_authenticated_user;

/// Forgot password endpoint
pub async fn forgot_password(
    state: web::Data<AppState>,
    request: web::Json<ForgotPasswordRequest>,
) -> ActixResult<HttpResponse> {
    info!("Password reset request received");

    // Generate reset token
    match state.auth.request_password_reset(&request.email).await {
        Ok(_reset_token) => {
            // TODO: Send email with reset token
            info!("Password reset token generated");
            Ok(HttpResponse::Ok().json(ApiResponse::success(())))
        }
        Err(e) => {
            // Don't reveal if email exists or not
            warn!("Password reset request failed: {}", e);
            Ok(HttpResponse::Ok().json(ApiResponse::success(())))
        }
    }
}

/// Reset password endpoint
pub async fn reset_password(
    state: web::Data<AppState>,
    request: web::Json<ResetPasswordRequest>,
) -> ActixResult<HttpResponse> {
    info!("Password reset with token");

    // Validate new password
    if let Err(e) = DataValidator::validate_password(&request.new_password) {
        return Ok(HttpResponse::Ok().json(ApiResponse::<()>::error_for_type(e.to_string())));
    }

    // Reset password
    match state
        .auth
        .reset_password(&request.token, &request.new_password)
        .await
    {
        Ok(()) => {
            info!("Password reset successfully");
            Ok(HttpResponse::Ok().json(ApiResponse::success(())))
        }
        Err(e) => {
            warn!("Password reset failed: {}", e);
            Ok(HttpResponse::Ok().json(ApiResponse::<()>::error_for_type(
                "Invalid or expired reset token".to_string(),
            )))
        }
    }
}

/// Change password endpoint
pub async fn change_password(
    state: web::Data<AppState>,
    req: HttpRequest,
    request: web::Json<ChangePasswordRequest>,
) -> ActixResult<HttpResponse> {
    info!("Password change request");

    // Get authenticated user
    let user = match get_authenticated_user(&req) {
        Some(user) => user,
        None => {
            return Ok(HttpResponse::Unauthorized()
                .json(ApiResponse::<()>::error("Unauthorized".to_string())));
        }
    };

    // Validate new password
    if let Err(e) = DataValidator::validate_password(&request.new_password) {
        return Ok(HttpResponse::Ok().json(ApiResponse::<()>::error_for_type(e.to_string())));
    }

    // Change password
    match state
        .auth
        .change_password(user.id(), &request.current_password, &request.new_password)
        .await
    {
        Ok(()) => {
            info!("Password changed successfully for user: {}", user.username);
            Ok(HttpResponse::Ok().json(ApiResponse::success(())))
        }
        Err(e) => {
            warn!("Password change failed: {}", e);
            Ok(HttpResponse::Ok().json(ApiResponse::<()>::error_for_type(e.to_string())))
        }
    }
}
