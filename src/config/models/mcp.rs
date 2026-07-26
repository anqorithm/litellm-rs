//! MCP gateway configuration (YAML config model)
//!
//! This is the YAML-deserialized MCP config for the gateway config file. The
//! runtime types live in [`crate::core::mcp::config`]; this model is translated
//! into them by [`GatewayMcpConfig::to_runtime_config`] during startup.
//!
//! ```yaml
//! mcp:
//!   servers:
//!     vertus_tools:
//!       url: "https://42.vertus.ai/mcp/"
//!       transport: http
//!       static_headers:
//!         X-Service-Key: "${VERTUS_SERVICE_KEY}"
//! ```

use super::*;
use crate::core::mcp::config::{AuthConfig, McpGatewayConfig, McpServerConfig};
use crate::core::mcp::transport::Transport;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Gateway MCP configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayMcpConfig {
    /// MCP servers keyed by server name.
    ///
    /// The key is the server name used to prefix aggregated tool names as
    /// `mcp_{server}__{tool}` on the `/mcp` JSON-RPC surface.
    #[serde(default)]
    pub servers: BTreeMap<String, GatewayMcpServerConfig>,
    /// When constructing the MCP gateway fails, keep serving traffic without
    /// the MCP surface instead of failing startup.
    ///
    /// Defaults to `false` so a configured-but-broken MCP section is surfaced
    /// at startup.
    #[serde(default)]
    pub allow_degraded: bool,
}

impl GatewayMcpConfig {
    /// Merge MCP configurations, with the overlay taking precedence per server.
    pub fn merge(mut self, other: Self) -> Self {
        for (name, server) in other.servers {
            self.servers.insert(name, server);
        }
        if other.allow_degraded {
            self.allow_degraded = other.allow_degraded;
        }
        self
    }

    /// Whether at least one server is configured and enabled.
    pub fn has_enabled_servers(&self) -> bool {
        self.servers.values().any(|server| server.enabled)
    }

    /// Translate into the runtime MCP gateway configuration.
    ///
    /// Disabled servers are dropped so they are never registered.
    pub fn to_runtime_config(&self) -> McpGatewayConfig {
        let servers = self
            .servers
            .iter()
            .filter(|(_, server)| server.enabled)
            .map(|(name, server)| (name.clone(), server.to_runtime_config(name)))
            .collect();

        McpGatewayConfig {
            servers,
            ..McpGatewayConfig::default()
        }
    }

    /// Validate the MCP configuration.
    pub fn validate(&self) -> Result<(), String> {
        for (name, server) in &self.servers {
            if name.trim().is_empty() {
                return Err("mcp.servers keys cannot be empty".to_string());
            }
            if name.contains("__") {
                return Err(format!(
                    "mcp.servers.{name} must not contain '__'; it would break the \
                     mcp_{{server}}__{{tool}} name prefix"
                ));
            }
            server.validate(name)?;
        }

        self.to_runtime_config()
            .validate()
            .map_err(|errors| format!("invalid mcp.servers: {}", errors.join("; ")))
    }
}

/// One MCP server entry in the gateway config file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayMcpServerConfig {
    /// Server endpoint URL. Must be `http(s)://` for the supported transports.
    pub url: String,
    /// Transport protocol. Only `http` and `sse` are served by the runtime.
    #[serde(default)]
    pub transport: Transport,
    /// Headers sent verbatim on every request to this server.
    #[serde(default)]
    pub static_headers: BTreeMap<String, String>,
    /// Client header names forwarded to this server.
    #[serde(default)]
    pub forward_headers: Vec<String>,
    /// Authentication applied to every request to this server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthConfig>,
    /// Per-request timeout in milliseconds.
    #[serde(default = "default_mcp_timeout_ms")]
    pub timeout_ms: u64,
    /// Whether this server is registered at startup.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Human-readable description of this server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Default for GatewayMcpServerConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            transport: Transport::default(),
            static_headers: BTreeMap::new(),
            forward_headers: Vec::new(),
            auth: None,
            timeout_ms: default_mcp_timeout_ms(),
            enabled: true,
            description: None,
        }
    }
}

impl GatewayMcpServerConfig {
    /// Translate into the runtime server configuration for `name`.
    pub fn to_runtime_config(&self, name: &str) -> McpServerConfig {
        McpServerConfig {
            name: name.to_string(),
            url: self.url.clone(),
            transport: self.transport,
            auth: self.auth.clone(),
            static_headers: self
                .static_headers
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
            forward_headers: self.forward_headers.clone(),
            timeout_ms: self.timeout_ms,
            enabled: self.enabled,
            description: self.description.clone(),
            ..McpServerConfig::default()
        }
    }

    fn validate(&self, name: &str) -> Result<(), String> {
        match self.transport {
            Transport::Http | Transport::Sse => {}
            Transport::Stdio | Transport::WebSocket => {
                return Err(format!(
                    "mcp.servers.{name}.transport={} is not served by the gateway runtime; \
                     use http or sse",
                    self.transport
                ));
            }
        }

        if self.timeout_ms == 0 {
            return Err(format!(
                "mcp.servers.{name}.timeout_ms must be greater than 0"
            ));
        }

        // The runtime skips headers it cannot represent, so reject them here
        // rather than silently dropping an auth header at request time.
        for (key, value) in &self.static_headers {
            if reqwest::header::HeaderName::from_bytes(key.as_bytes()).is_err() {
                return Err(format!(
                    "mcp.servers.{name}.static_headers has invalid header name '{key}'"
                ));
            }
            if reqwest::header::HeaderValue::from_str(value).is_err() {
                return Err(format!(
                    "mcp.servers.{name}.static_headers has invalid value for header '{key}'"
                ));
            }
        }

        for header in &self.forward_headers {
            if reqwest::header::HeaderName::from_bytes(header.as_bytes()).is_err() {
                return Err(format!(
                    "mcp.servers.{name}.forward_headers has invalid header name '{header}'"
                ));
            }
        }

        if let Some(auth) = &self.auth {
            auth.validate()
                .map_err(|error| format!("mcp.servers.{name}.auth is invalid: {error}"))?;
        }

        Ok(())
    }
}

fn default_mcp_timeout_ms() -> u64 {
    30_000
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::mcp::config::McpAuthType;

    fn vertus_tools_yaml() -> &'static str {
        r#"
servers:
  vertus_tools:
    url: "https://42.vertus.ai/mcp/"
    transport: http
    static_headers:
      X-Service-Key: "service-key-value"
"#
    }

    #[test]
    fn test_mcp_config_default_is_empty_and_disabled() {
        let config = GatewayMcpConfig::default();
        assert!(config.servers.is_empty());
        assert!(!config.allow_degraded);
        assert!(!config.has_enabled_servers());
    }

    #[test]
    fn test_mcp_config_deserializes_python_proxy_shape() {
        let config: GatewayMcpConfig =
            serde_yml::from_str(vertus_tools_yaml()).expect("mcp config should parse");

        let server = config
            .servers
            .get("vertus_tools")
            .expect("vertus_tools should be configured");
        assert_eq!(server.url, "https://42.vertus.ai/mcp/");
        assert_eq!(server.transport, Transport::Http);
        assert_eq!(
            server
                .static_headers
                .get("X-Service-Key")
                .map(String::as_str),
            Some("service-key-value")
        );
        assert_eq!(server.timeout_ms, 30_000);
        assert!(server.enabled);
        assert!(config.has_enabled_servers());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_mcp_config_rejects_unknown_fields() {
        let yaml = r#"
servers:
  tools:
    url: "https://42.vertus.ai/mcp/"
    unexpected: true
"#;
        assert!(serde_yml::from_str::<GatewayMcpConfig>(yaml).is_err());
    }

    #[test]
    fn test_mcp_config_deserializes_auth_section() {
        let yaml = r#"
servers:
  tools:
    url: "https://42.vertus.ai/mcp/"
    auth:
      type: bearer_token
      value: "token123"
"#;
        let config: GatewayMcpConfig = serde_yml::from_str(yaml).expect("auth config should parse");
        let auth = config.servers["tools"]
            .auth
            .as_ref()
            .expect("auth should be present");
        assert_eq!(auth.auth_type, McpAuthType::BearerToken);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_to_runtime_config_translates_name_and_headers() {
        let config: GatewayMcpConfig =
            serde_yml::from_str(vertus_tools_yaml()).expect("mcp config should parse");
        let runtime = config.to_runtime_config();

        let server = runtime
            .servers
            .get("vertus_tools")
            .expect("runtime server should exist");
        assert_eq!(server.name, "vertus_tools");
        assert_eq!(server.url, "https://42.vertus.ai/mcp/");
        assert_eq!(
            server
                .static_headers
                .get("X-Service-Key")
                .map(String::as_str),
            Some("service-key-value")
        );
        assert!(runtime.validate().is_ok());
    }

    #[test]
    fn test_to_runtime_config_drops_disabled_servers() {
        let yaml = r#"
servers:
  disabled_tools:
    url: "https://42.vertus.ai/mcp/"
    enabled: false
"#;
        let config: GatewayMcpConfig = serde_yml::from_str(yaml).expect("mcp config should parse");
        assert!(!config.has_enabled_servers());
        assert!(config.to_runtime_config().servers.is_empty());
    }

    #[test]
    fn test_validate_rejects_unsupported_transports() {
        for transport in ["stdio", "websocket"] {
            let yaml = format!(
                r#"
servers:
  tools:
    url: "https://42.vertus.ai/mcp/"
    transport: {transport}
"#
            );
            let config: GatewayMcpConfig =
                serde_yml::from_str(&yaml).expect("transport should parse");
            let error = config
                .validate()
                .expect_err("unsupported transport must be rejected");
            assert!(
                error.contains("not served by the gateway runtime"),
                "{error}"
            );
        }
    }

    #[test]
    fn test_validate_rejects_private_targets() {
        let yaml = r#"
servers:
  tools:
    url: "http://127.0.0.1:9000/mcp"
"#;
        let config: GatewayMcpConfig = serde_yml::from_str(yaml).expect("mcp config should parse");
        let error = config
            .validate()
            .expect_err("loopback target must be rejected");
        assert!(error.contains("private or reserved"), "{error}");
    }

    #[test]
    fn test_validate_rejects_server_name_with_tool_separator() {
        let yaml = r#"
servers:
  bad__name:
    url: "https://42.vertus.ai/mcp/"
"#;
        let config: GatewayMcpConfig = serde_yml::from_str(yaml).expect("mcp config should parse");
        let error = config
            .validate()
            .expect_err("server names must not contain the tool separator");
        assert!(error.contains("must not contain '__'"), "{error}");
    }

    #[test]
    fn test_validate_rejects_invalid_static_header() {
        let yaml = r#"
servers:
  tools:
    url: "https://42.vertus.ai/mcp/"
    static_headers:
      "Bad Header": "value"
"#;
        let config: GatewayMcpConfig = serde_yml::from_str(yaml).expect("mcp config should parse");
        let error = config
            .validate()
            .expect_err("invalid header name must be rejected");
        assert!(error.contains("invalid header name"), "{error}");
    }

    #[test]
    fn test_validate_rejects_zero_timeout() {
        let yaml = r#"
servers:
  tools:
    url: "https://42.vertus.ai/mcp/"
    timeout_ms: 0
"#;
        let config: GatewayMcpConfig = serde_yml::from_str(yaml).expect("mcp config should parse");
        let error = config
            .validate()
            .expect_err("zero timeout must be rejected");
        assert!(error.contains("timeout_ms"), "{error}");
    }

    #[test]
    fn test_merge_overlays_servers_and_allow_degraded() {
        let base: GatewayMcpConfig =
            serde_yml::from_str(vertus_tools_yaml()).expect("base should parse");
        let overlay: GatewayMcpConfig = serde_yml::from_str(
            r#"
allow_degraded: true
servers:
  vertus_tools:
    url: "https://42.vertus.ai/mcp/v2/"
  extra_tools:
    url: "https://tools.example.com/mcp"
"#,
        )
        .expect("overlay should parse");

        let merged = base.merge(overlay);
        assert!(merged.allow_degraded);
        assert_eq!(merged.servers.len(), 2);
        assert_eq!(
            merged.servers["vertus_tools"].url,
            "https://42.vertus.ai/mcp/v2/"
        );
        assert!(merged.servers.contains_key("extra_tools"));
    }

    #[test]
    fn test_merge_keeps_base_allow_degraded_when_overlay_is_default() {
        let base = GatewayMcpConfig {
            allow_degraded: true,
            ..GatewayMcpConfig::default()
        };
        let merged = base.merge(GatewayMcpConfig::default());
        assert!(merged.allow_degraded);
    }
}
