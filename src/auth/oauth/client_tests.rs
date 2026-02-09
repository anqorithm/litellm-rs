use super::*;
use crate::auth::oauth::config::OAuthProvider;

fn create_test_config() -> OAuthConfig {
    OAuthConfig::google("test_client_id", "https://app.example.com/callback")
        .with_client_secret("test_client_secret")
}

#[test]
fn test_oauth_client_creation() {
    let config = create_test_config();
    let client = OAuthClient::new(config);
    assert!(client.is_ok());
}

#[test]
fn test_oauth_client_invalid_config() {
    let mut config = create_test_config();
    config.client_id = String::new();
    let client = OAuthClient::new(config);
    assert!(client.is_err());
}

#[test]
fn test_authorization_url_generation() {
    let config = create_test_config();
    let client = OAuthClient::new(config).unwrap();
    let (url, state) = client.get_authorization_url();

    assert!(url.contains("accounts.google.com"));
    assert!(url.contains("client_id=test_client_id"));
    assert!(url.contains("redirect_uri="));
    assert!(url.contains(&format!("state={}", state.state)));
    assert!(url.contains("response_type=code"));
}

#[test]
fn test_authorization_url_with_pkce() {
    let config = create_test_config();
    let client = OAuthClient::new(config).unwrap();
    let (url, state) = client.get_authorization_url();

    assert!(url.contains("code_challenge="));
    assert!(url.contains("code_challenge_method=S256"));
    assert!(state.code_verifier.is_some());
}

#[test]
fn test_authorization_url_without_pkce() {
    let config = OAuthConfig::github("test_client_id", "https://app.example.com/callback")
        .with_client_secret("test_secret");
    let client = OAuthClient::new(config).unwrap();
    let (url, state) = client.get_authorization_url();

    assert!(!url.contains("code_challenge="));
    assert!(state.code_verifier.is_none());
}

#[test]
fn test_authorization_url_with_extra_params() {
    let config = create_test_config()
        .with_param("prompt", "consent")
        .with_param("access_type", "offline");
    let client = OAuthClient::new(config).unwrap();
    let (url, _) = client.get_authorization_url();

    assert!(url.contains("prompt=consent"));
    assert!(url.contains("access_type=offline"));
}

#[test]
fn test_callback_validation_success() {
    let config = create_test_config();
    let client = OAuthClient::new(config).unwrap();
    let (_, state) = client.get_authorization_url();

    let params = CallbackParams {
        code: Some("auth_code".to_string()),
        state: Some(state.state.clone()),
        error: None,
        error_description: None,
    };

    assert!(client.validate_callback(&params, &state).is_ok());
}

#[test]
fn test_callback_validation_error() {
    let config = create_test_config();
    let client = OAuthClient::new(config).unwrap();
    let (_, state) = client.get_authorization_url();

    let params = CallbackParams {
        code: None,
        state: Some(state.state.clone()),
        error: Some("access_denied".to_string()),
        error_description: Some("User denied access".to_string()),
    };

    let result = client.validate_callback(&params, &state);
    assert!(result.is_err());
}

#[test]
fn test_callback_validation_state_mismatch() {
    let config = create_test_config();
    let client = OAuthClient::new(config).unwrap();
    let (_, state) = client.get_authorization_url();

    let params = CallbackParams {
        code: Some("auth_code".to_string()),
        state: Some("wrong_state".to_string()),
        error: None,
        error_description: None,
    };

    let result = client.validate_callback(&params, &state);
    assert!(result.is_err());
}

#[test]
fn test_callback_validation_expired_state() {
    let config = create_test_config();
    let client = OAuthClient::new(config).unwrap();
    let (_, mut state) = client.get_authorization_url();

    // Make state expired
    state.created_at = chrono::Utc::now() - chrono::Duration::seconds(700);

    let params = CallbackParams {
        code: Some("auth_code".to_string()),
        state: Some(state.state.clone()),
        error: None,
        error_description: None,
    };

    let result = client.validate_callback(&params, &state);
    assert!(result.is_err());
}

#[test]
fn test_parse_user_info_google() {
    let config = create_test_config();
    let client = OAuthClient::new(config).unwrap();

    let json = r#"{
        "sub": "123456789",
        "email": "user@example.com",
        "email_verified": true,
        "name": "Test User",
        "picture": "https://example.com/photo.jpg"
    }"#;

    let user_info = client.parse_user_info(json).unwrap();
    assert_eq!(user_info.id, "123456789");
    assert_eq!(user_info.email, "user@example.com");
    assert_eq!(user_info.name, Some("Test User".to_string()));
    assert!(user_info.email_verified);
}

#[test]
fn test_parse_user_info_github() {
    let config = OAuthConfig::github("test_id", "https://app.example.com/callback");
    let client = OAuthClient::new(config).unwrap();

    let json = r#"{
        "id": 12345,
        "email": "user@example.com",
        "login": "testuser",
        "avatar_url": "https://github.com/avatar.jpg"
    }"#;

    let user_info = client.parse_user_info(json).unwrap();
    assert_eq!(user_info.id, "12345");
    assert_eq!(user_info.email, "user@example.com");
    assert_eq!(user_info.name, Some("testuser".to_string()));
    assert_eq!(
        user_info.picture,
        Some("https://github.com/avatar.jpg".to_string())
    );
}

#[test]
fn test_parse_token_response_json() {
    let config = create_test_config();
    let client = OAuthClient::new(config).unwrap();

    let json = r#"{
        "access_token": "access123",
        "token_type": "Bearer",
        "expires_in": 3600,
        "refresh_token": "refresh456",
        "scope": "openid email"
    }"#;

    let response = client.parse_token_response(json).unwrap();
    assert_eq!(response.access_token, "access123");
    assert_eq!(response.token_type, "Bearer");
    assert_eq!(response.expires_in, 3600);
    assert_eq!(response.refresh_token, Some("refresh456".to_string()));
}

#[test]
fn test_parse_token_response_urlencoded() {
    let config = OAuthConfig::github("test_id", "https://app.example.com/callback");
    let client = OAuthClient::new(config).unwrap();

    let body = "access_token=access123&token_type=bearer&scope=read%3Auser";

    let response = client.parse_token_response(body).unwrap();
    assert_eq!(response.access_token, "access123");
}

#[test]
fn test_logout_url_generation() {
    let config = create_test_config();
    let client = OAuthClient::new(config).unwrap();

    let url = client.get_logout_url(None, None);
    assert!(url.is_some());

    let url_with_params =
        client.get_logout_url(Some("id_token_123"), Some("https://app.example.com"));
    assert!(url_with_params.is_some());
    let url = url_with_params.unwrap();
    assert!(url.contains("id_token_hint="));
    assert!(url.contains("post_logout_redirect_uri="));
}

#[test]
fn test_oauth_client_debug() {
    let config = create_test_config();
    let client = OAuthClient::new(config).unwrap();
    let debug_str = format!("{:?}", client);
    assert!(debug_str.contains("OAuthClient"));
    assert!(debug_str.contains("Google"));
}

#[test]
fn test_extract_user_id_various_formats() {
    let config = create_test_config();
    let client = OAuthClient::new(config).unwrap();

    // OIDC standard 'sub'
    let json1 = serde_json::json!({"sub": "user123"});
    assert_eq!(client.extract_user_id(&json1).unwrap(), "user123");

    // GitHub style numeric 'id'
    let json2 = serde_json::json!({"id": 12345});
    assert_eq!(client.extract_user_id(&json2).unwrap(), "12345");

    // Microsoft 'oid'
    let json3 = serde_json::json!({"oid": "guid-123-456"});
    assert_eq!(client.extract_user_id(&json3).unwrap(), "guid-123-456");
}
