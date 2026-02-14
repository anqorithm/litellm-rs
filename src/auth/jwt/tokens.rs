//! Specialized token operations (password reset, email verification, invitations)

use super::types::{Claims, JwtHandler, TokenType};
use crate::utils::error::gateway_error::{GatewayError, Result};
use jsonwebtoken::{Header, Validation, decode, encode};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;
use uuid::Uuid;

impl JwtHandler {
    /// Create a password reset token
    pub async fn create_password_reset_token(&self, user_id: Uuid) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| GatewayError::internal(format!("System time error: {}", e)))?
            .as_secs();

        let claims = Claims {
            sub: user_id,
            iat: now,
            exp: now + 3600, // 1 hour expiration
            iss: self.issuer.clone(),
            aud: "password_reset".to_string(),
            jti: Uuid::new_v4().to_string(),
            role: "".to_string(),
            permissions: vec![],
            team_id: None,
            session_id: None,
            token_type: TokenType::PasswordReset,
        };

        let header = Header::new(self.algorithm);
        let token = encode(&header, &claims, &self.encoding_key).map_err(GatewayError::Jwt)?;

        debug!("Created password reset token for user: {}", user_id);
        Ok(token)
    }

    /// Verify a password reset token
    pub async fn verify_password_reset_token(&self, token: &str) -> Result<Uuid> {
        let mut validation = Validation::new(self.algorithm);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&["password_reset"]);

        let token_data =
            decode::<Claims>(token, &self.decoding_key, &validation).map_err(GatewayError::Jwt)?;

        if !matches!(token_data.claims.token_type, TokenType::PasswordReset) {
            return Err(GatewayError::auth("Invalid token type for password reset"));
        }

        Ok(token_data.claims.sub)
    }

    /// Create an email verification token
    pub async fn create_email_verification_token(&self, user_id: Uuid) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| GatewayError::internal(format!("System time error: {}", e)))?
            .as_secs();

        let claims = Claims {
            sub: user_id,
            iat: now,
            exp: now + 86400, // 24 hours expiration
            iss: self.issuer.clone(),
            aud: "email_verification".to_string(),
            jti: Uuid::new_v4().to_string(),
            role: "".to_string(),
            permissions: vec![],
            team_id: None,
            session_id: None,
            token_type: TokenType::EmailVerification,
        };

        let header = Header::new(self.algorithm);
        let token = encode(&header, &claims, &self.encoding_key).map_err(GatewayError::Jwt)?;

        debug!("Created email verification token for user: {}", user_id);
        Ok(token)
    }

    /// Verify an email verification token
    pub async fn verify_email_verification_token(&self, token: &str) -> Result<Uuid> {
        let mut validation = Validation::new(self.algorithm);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&["email_verification"]);

        let token_data =
            decode::<Claims>(token, &self.decoding_key, &validation).map_err(GatewayError::Jwt)?;

        if !matches!(token_data.claims.token_type, TokenType::EmailVerification) {
            return Err(GatewayError::auth(
                "Invalid token type for email verification",
            ));
        }

        Ok(token_data.claims.sub)
    }

    /// Create an invitation token
    pub async fn create_invitation_token(
        &self,
        user_id: Uuid,
        team_id: Uuid,
        role: String,
    ) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| GatewayError::internal(format!("System time error: {}", e)))?
            .as_secs();

        let claims = Claims {
            sub: user_id,
            iat: now,
            exp: now + 604800, // 7 days expiration
            iss: self.issuer.clone(),
            aud: "invitation".to_string(),
            jti: Uuid::new_v4().to_string(),
            role,
            permissions: vec![],
            team_id: Some(team_id),
            session_id: None,
            token_type: TokenType::Invitation,
        };

        let header = Header::new(self.algorithm);
        let token = encode(&header, &claims, &self.encoding_key).map_err(GatewayError::Jwt)?;

        debug!(
            "Created invitation token for user: {} team: {}",
            user_id, team_id
        );
        Ok(token)
    }

    /// Verify an invitation token
    pub async fn verify_invitation_token(&self, token: &str) -> Result<(Uuid, Uuid, String)> {
        let mut validation = Validation::new(self.algorithm);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&["invitation"]);

        let token_data =
            decode::<Claims>(token, &self.decoding_key, &validation).map_err(GatewayError::Jwt)?;

        if !matches!(token_data.claims.token_type, TokenType::Invitation) {
            return Err(GatewayError::auth("Invalid token type for invitation"));
        }

        let team_id = token_data
            .claims
            .team_id
            .ok_or_else(|| GatewayError::auth("Missing team ID in invitation token"))?;

        Ok((token_data.claims.sub, team_id, token_data.claims.role))
    }
}
