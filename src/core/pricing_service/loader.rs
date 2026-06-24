//! Data loading functionality for the pricing service

use super::service::PricingService;
use super::types::LiteLLMModelInfo;
use crate::core::pricing::{embedded_default_pricing_models, parse_litellm_pricing_json};
use crate::utils::error::gateway_error::{GatewayError, Result};
use std::collections::HashMap;
use std::time::Duration;
use tracing::debug;

impl PricingService {
    /// Initialize pricing data (load from URL or local file)
    pub async fn initialize(&self) -> Result<()> {
        self.refresh_pricing_data().await
    }

    /// Load pricing data from URL
    pub(super) async fn load_from_url(&self) -> Result<HashMap<String, LiteLLMModelInfo>> {
        let response = self
            .http_client
            .get(&self.pricing_url)
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| GatewayError::network(format!("Failed to fetch pricing data: {}", e)))?;

        if !response.status().is_success() {
            return Err(GatewayError::network(format!(
                "HTTP {}: Failed to fetch pricing data",
                response.status()
            )));
        }

        let text = response
            .text()
            .await
            .map_err(|e| GatewayError::network(format!("Failed to read response: {}", e)))?;

        let data = parse_litellm_pricing_json(&text)
            .map_err(|e| GatewayError::parsing(format!("Failed to parse pricing JSON: {}", e)))?;

        debug!("Loaded {} models from URL", data.len());
        Ok(data)
    }

    /// Load pricing data from local file
    pub(super) async fn load_from_file(&self) -> Result<HashMap<String, LiteLLMModelInfo>> {
        let content = tokio::fs::read_to_string(&self.pricing_url)
            .await
            .map_err(GatewayError::Io)?;

        let data = parse_litellm_pricing_json(&content)
            .map_err(|e| GatewayError::parsing(format!("Failed to parse pricing JSON: {}", e)))?;

        debug!("Loaded {} models from file", data.len());
        Ok(data)
    }

    /// Load bundled default pricing data.
    pub(super) fn load_from_embedded_default(&self) -> Result<HashMap<String, LiteLLMModelInfo>> {
        let data = embedded_default_pricing_models()
            .map_err(|e| GatewayError::parsing(format!("Failed to parse pricing JSON: {}", e)))?;

        debug!("Loaded {} models from embedded default pricing", data.len());
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::pricing_service::DEFAULT_PRICING_SOURCE;
    use std::fs;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    fn pricing_json(model: &str, provider: &str) -> String {
        format!(
            r#"{{
                "{model}": {{
                    "input_cost_per_token": 0.000001,
                    "output_cost_per_token": 0.000002,
                    "litellm_provider": "{provider}",
                    "mode": "chat"
                }}
            }}"#
        )
    }

    #[tokio::test]
    async fn default_pricing_source_loads_embedded_data() -> Result<()> {
        let service = PricingService::new(Some(DEFAULT_PRICING_SOURCE.to_string()));

        service.initialize().await?;

        assert!(service.get_model_info("gpt-4o").is_some());
        Ok(())
    }

    #[tokio::test]
    async fn explicit_relative_pricing_source_uses_filesystem_path() -> Result<()> {
        let temp_dir = tempfile::tempdir_in(".").map_err(GatewayError::Io)?;
        let file_path = temp_dir.path().join("custom-prices.json");
        fs::write(&file_path, pricing_json("custom-priced-model", "custom"))
            .map_err(GatewayError::Io)?;
        let relative_path = if file_path.is_absolute() {
            file_path
                .strip_prefix(std::env::current_dir().map_err(GatewayError::Io)?)
                .map_err(|error| GatewayError::Config(error.to_string()))?
                .to_path_buf()
        } else {
            file_path
        }
        .to_string_lossy()
        .to_string();
        let service = PricingService::new(Some(relative_path));

        service.initialize().await?;

        assert_eq!(
            service
                .get_model_info("custom-priced-model")
                .map(|info| info.litellm_provider),
            Some("custom".to_string())
        );
        assert!(service.get_model_info("gpt-4o").is_none());
        Ok(())
    }

    #[tokio::test]
    async fn explicit_remote_pricing_parse_failure_is_returned() -> Result<()> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(GatewayError::Io)?;
        let addr = listener.local_addr().map_err(GatewayError::Io)?;
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await?;
            let mut request_buffer = [0_u8; 1024];
            let bytes_read = stream.read(&mut request_buffer).await?;
            if bytes_read == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "test server received an empty request",
                ));
            }
            let body = r#"{
                "remote-only-malformed": {
                    "max_tokens": 2000000.5,
                    "input_cost_per_token": 0.000001,
                    "output_cost_per_token": 0.000002,
                    "litellm_provider": "openai",
                    "mode": "chat"
                }
            }"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).await?;
            Ok::<(), std::io::Error>(())
        });

        let service = PricingService::new(Some(format!("http://{addr}/prices.json")));

        let result = service.initialize().await;
        server
            .await
            .map_err(|error| GatewayError::internal(format!("test server failed: {error}")))?
            .map_err(GatewayError::Io)?;

        assert!(result.is_err());
        assert!(service.get_model_info("gpt-4o").is_none());
        assert!(service.get_model_info("remote-only-malformed").is_none());
        Ok(())
    }
}
