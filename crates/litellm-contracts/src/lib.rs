pub mod openai;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderConfig {
    pub name: String,
    pub provider_type: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub models: Vec<String>,
    pub settings: HashMap<String, serde_json::Value>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GatewayConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub providers: Vec<ProviderConfig>,
    pub default_model: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config file {path}: {message}")]
    Read { path: String, message: String },
    #[error("failed to parse config file {path}: {message}")]
    Parse { path: String, message: String },
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8000
}

impl GatewayConfig {
    pub fn from_yaml_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path_ref = path.as_ref();
        let path_str = path_ref.display().to_string();
        let raw = std::fs::read_to_string(path_ref).map_err(|err| ConfigError::Read {
            path: path_str.clone(),
            message: err.to_string(),
        })?;
        serde_yaml::from_str(&raw).map_err(|err| ConfigError::Parse {
            path: path_str,
            message: err.to_string(),
        })
    }
}
