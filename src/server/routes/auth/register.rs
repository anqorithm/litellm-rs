//! User registration endpoint

use crate::server::routes::ApiResponse;
use crate::server::state::AppState;
use crate::utils::auth::crypto::password::hash_password;
use crate::utils::data::validation::DataValidator;
use actix_web::{HttpResponse, Result as ActixResult, web};
use tracing::{error, info};

use super::models::{RegisterRequest, RegisterResponse};

/// User registration endpoint
pub async fn register(
    state: web::Data<AppState>,
    request: web::Json<RegisterRequest>,
) -> ActixResult<HttpResponse> {
    info!("User registration attempt: {}", request.username);

    // Validate input
    if let Err(e) = DataValidator::validate_username(&request.username) {
        return Ok(
            HttpResponse::BadRequest().json(ApiResponse::<()>::error_for_type(e.to_string()))
        );
    }

    if let Err(e) = crate::utils::config::helpers::ConfigValidator::validate_email(&request.email) {
        return Ok(
            HttpResponse::BadRequest().json(ApiResponse::<()>::error_for_type(e.to_string()))
        );
    }

    if let Err(e) = DataValidator::validate_password(&request.password) {
        return Ok(
            HttpResponse::BadRequest().json(ApiResponse::<()>::error_for_type(e.to_string()))
        );
    }

    // Check if user already exists
    match state
        .storage
        .database
        .find_user_by_username(&request.username)
        .await
    {
        Ok(Some(_)) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                "Username already exists".to_string(),
            )));
        }
        Ok(None) => {} // Continue with registration
        Err(e) => {
            error!("Failed to check existing user: {}", e);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Database error".to_string())));
        }
    }

    // Check if email already exists
    match state
        .storage
        .database
        .find_user_by_email(&request.email)
        .await
    {
        Ok(Some(_)) => {
            return Ok(HttpResponse::BadRequest()
                .json(ApiResponse::<()>::error("Email already exists".to_string())));
        }
        Ok(None) => {} // Continue with registration
        Err(e) => {
            error!("Failed to check existing email: {}", e);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Database error".to_string())));
        }
    }

    // Hash password
    let password_hash = match hash_password(&request.password) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Failed to hash password: {}", e);
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Password hashing failed".to_string(),
                )),
            );
        }
    };

    // Create user
    let user = crate::core::models::user::types::User::new(
        request.username.clone(),
        request.email.clone(),
        password_hash,
    );

    // Store user in database
    match state.storage.database.create_user(&user).await {
        Ok(created_user) => {
            info!("User registered successfully: {}", created_user.username);

            let response = RegisterResponse {
                user_id: created_user.id(),
                username: created_user.username,
                email: created_user.email,
                message: "Registration successful. Please verify your email.".to_string(),
            };

            Ok(HttpResponse::Created().json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Failed to create user: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("User creation failed".to_string())))
        }
    }
}
