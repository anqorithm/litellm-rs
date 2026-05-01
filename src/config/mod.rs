//! Configuration management for the Gateway
//!
//! This module handles loading, validation, and management of all gateway configuration.
//! Canonical server-side models live under `crate::config::models::*`.

pub mod builder;
pub mod models;
pub mod validation;

pub use validation::Validate;

use crate::config::models::auth::AuthConfig;
use crate::config::models::gateway::GatewayConfig;
use crate::config::models::monitoring::MonitoringConfig;
use crate::config::models::provider::ProviderConfig;
use crate::config::models::router::GatewayRouterConfig;
use crate::config::models::server::ServerConfig;
use crate::config::models::storage::StorageConfig;
use crate::utils::error::gateway_error::{GatewayError, Result};
use regex::Regex;
use std::collections::BTreeSet;
use std::path::Path;
use tracing::{debug, info};

/// Canonical alias for gateway server runtime configuration.
pub type GatewayServerConfig = crate::config::models::server::ServerConfig;
/// Canonical alias for gateway provider runtime configuration.
pub type GatewayProviderConfig = crate::config::models::provider::ProviderConfig;

/// Substitutes `${VAR_NAME}` and shell-style `$VAR_NAME` patterns in a string
/// with corresponding environment variable values.
///
/// Missing braced variables are treated as configuration errors so explicit
/// placeholders cannot accidentally reach runtime as literal secrets or URLs.
/// Bare `$VAR_NAME` keeps legacy convenience substitution, but unresolved bare
/// tokens are left literal to avoid rejecting ordinary dollar-containing values.
fn substitute_env_vars(input: &str) -> Result<String> {
    let env_re =
        Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}|(^|[^A-Za-z0-9_$])\$([A-Za-z_][A-Za-z0-9_]*)")
            .expect("static regex is valid");
    let mut missing_vars = BTreeSet::new();
    let mut substituted = String::with_capacity(input.len());

    for line in input.split_inclusive('\n') {
        let (line_body, line_ending) = split_line_ending(line);
        let (config_text, comment) = split_yaml_comment(line_body);
        substituted.push_str(&substitute_env_vars_in_segment(
            config_text,
            &env_re,
            &mut missing_vars,
        ));
        substituted.push_str(comment);
        substituted.push_str(line_ending);
    }

    if !missing_vars.is_empty() {
        let missing = missing_vars.into_iter().collect::<Vec<_>>().join(", ");
        return Err(GatewayError::Config(format!(
            "Missing environment variables referenced by config: {}",
            missing
        )));
    }

    Ok(substituted)
}

fn substitute_env_vars_in_segment(
    segment: &str,
    env_re: &Regex,
    missing_vars: &mut BTreeSet<String>,
) -> String {
    env_re
        .replace_all(segment, |caps: &regex::Captures<'_>| {
            if let Some(var_match) = caps.get(1) {
                let var_name = var_match.as_str();
                return match std::env::var(var_name) {
                    Ok(val) => val,
                    Err(_) => {
                        missing_vars.insert(var_name.to_string());
                        caps[0].to_string()
                    }
                };
            }

            let prefix = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let var_name = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            match std::env::var(var_name) {
                Ok(val) => format!("{}{}", prefix, val),
                Err(_) => caps[0].to_string(),
            }
        })
        .into_owned()
}

fn split_line_ending(line: &str) -> (&str, &str) {
    if let Some(body) = line.strip_suffix("\r\n") {
        (body, "\r\n")
    } else if let Some(body) = line.strip_suffix('\n') {
        (body, "\n")
    } else {
        (line, "")
    }
}

fn split_yaml_comment(line: &str) -> (&str, &str) {
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut double_quote_escaped = false;
    let mut single_quote_escaped = false;
    let mut previous_char = None;

    for (idx, ch) in line.char_indices() {
        if in_double_quote {
            if double_quote_escaped {
                double_quote_escaped = false;
            } else if ch == '\\' {
                double_quote_escaped = true;
            } else if ch == '"' {
                in_double_quote = false;
            }
            previous_char = Some(ch);
            continue;
        }

        if in_single_quote {
            if single_quote_escaped {
                single_quote_escaped = false;
            } else if ch == '\'' && line[idx + ch.len_utf8()..].starts_with('\'') {
                single_quote_escaped = true;
            } else if ch == '\'' {
                in_single_quote = false;
            }
            previous_char = Some(ch);
            continue;
        }

        match ch {
            '\'' => in_single_quote = true,
            '"' => in_double_quote = true,
            '#' if previous_char.is_none_or(char::is_whitespace) => return line.split_at(idx),
            _ => {}
        }

        previous_char = Some(ch);
    }

    (line, "")
}

/// Main configuration struct for the Gateway
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Gateway configuration
    pub gateway: GatewayConfig,
}

impl Config {
    /// Load configuration from file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        info!("Loading configuration from: {:?}", path);

        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| GatewayError::Config(format!("Failed to read config file: {}", e)))?;

        let content = substitute_env_vars(&content)?;

        let gateway: GatewayConfig = serde_yml::from_str(&content)
            .map_err(|e| GatewayError::Config(format!("Failed to parse config: {}", e)))?;

        let config = Self { gateway };

        // Configuration
        config.validate()?;

        debug!("Configuration loaded successfully");
        Ok(config)
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        info!("Loading configuration from environment variables");

        let gateway = GatewayConfig::from_env()?;
        let config = Self { gateway };

        config.validate()?;
        Ok(config)
    }

    /// Get server configuration
    pub fn server(&self) -> &ServerConfig {
        &self.gateway.server
    }

    /// Get providers configuration
    pub fn providers(&self) -> &[ProviderConfig] {
        &self.gateway.providers
    }

    /// Get router settings
    pub fn router(&self) -> &GatewayRouterConfig {
        &self.gateway.router
    }

    /// Get storage configuration
    pub fn storage(&self) -> &StorageConfig {
        &self.gateway.storage
    }

    /// Get auth configuration
    pub fn auth(&self) -> &AuthConfig {
        &self.gateway.auth
    }

    /// Get monitoring configuration
    pub fn monitoring(&self) -> &MonitoringConfig {
        &self.gateway.monitoring
    }

    /// Validate the entire configuration
    pub fn validate(&self) -> Result<()> {
        debug!("Validating configuration");

        // Single validation entry-point via Validate trait implementations.
        validation::Validate::validate(&self.gateway)
            .map_err(|e| GatewayError::Config(format!("Gateway config error: {}", e)))?;

        // Warn about insecure configurations
        crate::config::models::auth::warn_insecure_config(&self.gateway.auth);

        debug!("Configuration validation completed");
        Ok(())
    }

    /// Merge with another configuration (other takes precedence)
    pub fn merge(mut self, other: Self) -> Self {
        self.gateway = self.gateway.merge(other.gateway);
        self
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.gateway)
            .map_err(|e| GatewayError::Config(format!("Failed to serialize config to JSON: {}", e)))
    }

    /// Convert to YAML string
    pub fn to_yaml(&self) -> Result<String> {
        serde_yml::to_string(&self.gateway)
            .map_err(|e| GatewayError::Config(format!("Failed to serialize config to YAML: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_substitute_env_vars_braces() {
        // SAFETY: test-only mutation of env, not run in parallel with other tests touching this var
        unsafe { std::env::set_var("TEST_HOST", "example.com") };
        let result = substitute_env_vars("host: ${TEST_HOST}").unwrap();
        assert_eq!(result, "host: example.com");
        unsafe { std::env::remove_var("TEST_HOST") };
    }

    #[test]
    fn test_substitute_env_vars_bare() {
        // SAFETY: test-only mutation of env, not run in parallel with other tests touching this var
        unsafe { std::env::set_var("TEST_PORT", "9000") };
        let result = substitute_env_vars("port: $TEST_PORT").unwrap();
        assert_eq!(result, "port: 9000");
        unsafe { std::env::remove_var("TEST_PORT") };
    }

    #[test]
    fn test_substitute_env_vars_missing_fails_with_var_name() {
        // SAFETY: test-only removal, unique var name avoids interference
        unsafe { std::env::remove_var("DEFINITELY_NOT_SET_XYZ") };
        let err = substitute_env_vars("key: ${DEFINITELY_NOT_SET_XYZ}").unwrap_err();
        assert!(err.to_string().contains("DEFINITELY_NOT_SET_XYZ"));
    }

    #[test]
    fn test_substitute_env_vars_missing_lists_each_var_once() {
        // SAFETY: test-only removal, unique var names avoid interference
        unsafe {
            std::env::remove_var("DEFINITELY_NOT_SET_A");
            std::env::remove_var("DEFINITELY_NOT_SET_B");
        };
        let err = substitute_env_vars(
            "a: ${DEFINITELY_NOT_SET_A}\nb: ${DEFINITELY_NOT_SET_B}\nagain: ${DEFINITELY_NOT_SET_A}",
        )
        .unwrap_err()
        .to_string();

        assert!(err.contains("DEFINITELY_NOT_SET_A"));
        assert!(err.contains("DEFINITELY_NOT_SET_B"));
        assert_eq!(err.matches("DEFINITELY_NOT_SET_A").count(), 1);
    }

    #[test]
    fn test_substitute_env_vars_unresolved_bare_is_literal() {
        // SAFETY: test-only removal, unique var name avoids interference
        unsafe { std::env::remove_var("DEFINITELY_NOT_SET_LITERAL_TOKEN") };
        let result =
            substitute_env_vars("password: pa$word\nkey: $DEFINITELY_NOT_SET_LITERAL_TOKEN")
                .unwrap();

        assert_eq!(
            result,
            "password: pa$word\nkey: $DEFINITELY_NOT_SET_LITERAL_TOKEN"
        );
    }

    #[test]
    fn test_substitute_env_vars_ignores_yaml_comments() {
        // SAFETY: test-only env mutations use unique var names
        unsafe {
            std::env::set_var("COMMENT_REAL_VALUE", "ok");
            std::env::remove_var("DEFINITELY_NOT_SET_COMMENT_ONLY_A");
            std::env::remove_var("DEFINITELY_NOT_SET_COMMENT_ONLY_B");
        };

        let result = substitute_env_vars(
            "# set ${DEFINITELY_NOT_SET_COMMENT_ONLY_A}\nkey: ${COMMENT_REAL_VALUE} # ${DEFINITELY_NOT_SET_COMMENT_ONLY_B}\nurl: \"https://example.test/#fragment\"\nsingle: 'it''s # literal'",
        )
        .unwrap();

        assert_eq!(
            result,
            "# set ${DEFINITELY_NOT_SET_COMMENT_ONLY_A}\nkey: ok # ${DEFINITELY_NOT_SET_COMMENT_ONLY_B}\nurl: \"https://example.test/#fragment\"\nsingle: 'it''s # literal'"
        );
        unsafe { std::env::remove_var("COMMENT_REAL_VALUE") };
    }

    #[test]
    fn test_substitute_env_vars_does_not_expand_env_value_again() {
        // SAFETY: test-only env mutations use unique var names
        unsafe {
            std::env::set_var("OUTER_SECRET_WITH_DOLLAR", "pa$INNER_SECRET_TOKEN");
            std::env::set_var("INNER_SECRET_TOKEN", "expanded");
        };

        let result =
            substitute_env_vars("secret: ${OUTER_SECRET_WITH_DOLLAR}\nnext: $INNER_SECRET_TOKEN")
                .unwrap();

        assert_eq!(result, "secret: pa$INNER_SECRET_TOKEN\nnext: expanded");
        unsafe {
            std::env::remove_var("OUTER_SECRET_WITH_DOLLAR");
            std::env::remove_var("INNER_SECRET_TOKEN");
        };
    }

    #[tokio::test]
    async fn test_config_from_file() {
        let config_content = r#"
server:
  host: "127.0.0.1"
  port: 8080
  workers: 4

providers:
  - name: "openai"
    provider_type: "openai"
    api_key: "test-key"
    base_url: "https://api.openai.com/v1"

router:
  strategy: "round_robin"
  circuit_breaker:
    failure_threshold: 5
    recovery_timeout: 30

storage:
  database:
    url: "postgresql://localhost/gateway"
  redis:
    url: "redis://localhost:6379"

auth:
  jwt_secret: "TestSecretThatIsAtLeast32CharsLong123!"
  api_key_header: "Authorization"

monitoring:
  metrics:
    enabled: true
    port: 9090
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();

        let config = Config::from_file(temp_file.path()).await.unwrap();

        assert_eq!(config.server().host, "127.0.0.1");
        assert_eq!(config.server().port, 8080);
        assert_eq!(config.providers().len(), 1);
        assert_eq!(config.providers()[0].name, "openai");
    }

    #[tokio::test]
    async fn test_config_from_file_rejects_missing_env_var() {
        let config_content = r#"
server:
  host: "127.0.0.1"
  port: 8080

providers:
  - name: "openai"
    provider_type: "openai"
    api_key: "${DEFINITELY_NOT_SET_CONFIG_FILE_VAR}"

router:
  strategy: "round_robin"

storage:
  database:
    url: "postgresql://localhost/gateway"

auth:
  jwt_secret: "TestSecretThatIsAtLeast32CharsLong123!"

monitoring:
  metrics:
    enabled: true
"#;

        unsafe { std::env::remove_var("DEFINITELY_NOT_SET_CONFIG_FILE_VAR") };

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();

        let err = Config::from_file(temp_file.path()).await.unwrap_err();

        assert!(
            err.to_string()
                .contains("DEFINITELY_NOT_SET_CONFIG_FILE_VAR")
        );
    }

    #[test]
    fn test_gateway_config_rejects_unknown_top_level_field() {
        let err = serde_yml::from_str::<GatewayConfig>(
            r#"
schema_version: "1.0"
server: {}
providers: []
router: {}
storage:
  database:
    url: "postgresql://localhost/test"
  redis:
    url: "redis://localhost:6379"
auth: {}
monitoring: {}
typo_field: true
"#,
        )
        .unwrap_err()
        .to_string();

        assert!(
            err.contains("unknown field") || err.contains("typo_field"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_gateway_yaml_example_matches_config_schema() {
        let example_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("config/gateway.yaml.example");
        let content = std::fs::read_to_string(example_path).unwrap();
        let content = content
            .replace("${OPENAI_API_KEY}", "sk-test-openai")
            .replace("${ANTHROPIC_API_KEY}", "sk-ant-test")
            .replace(
                "${LITELLM_JWT_SECRET}",
                "StrongJwtSecretWithMixedCaseAndNumbers1234!",
            );

        let gateway: GatewayConfig = serde_yml::from_str(&content).unwrap();
        Config { gateway }.validate().unwrap();
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();

        let json = config.to_json().unwrap();
        assert!(!json.is_empty());

        let yaml = config.to_yaml().unwrap();
        assert!(!yaml.is_empty());
    }
}
