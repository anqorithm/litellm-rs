//! Secret Managers
//!
//! This module provides implementations for secret management across different backends.
//!
//! # Available Backends
//!
//! - **Environment Variables** (`EnvSecretManager`): Read secrets from environment variables
//! - **File-based** (`FileSecretManager`): Read secrets from files (for development)
//! - **AWS Secrets Manager** (`AwsSecretManager`): AWS Secrets Manager integration (requires `aws-secrets` feature)
//! - **Google Cloud Secret Manager** (`GcpSecretManager`): GCP Secret Manager integration (requires `gcp-secrets` feature)
//! - **Azure Key Vault** (`AzureSecretManager`): Azure Key Vault integration (requires `azure-secrets` feature)
//! - **HashiCorp Vault** (`VaultSecretManager`): HashiCorp Vault integration (requires `vault-secrets` feature)
//!
//! # Usage
//!
//! ```rust,ignore
//! use litellm_rs::core::secret_managers::{SecretManagerRegistry, EnvSecretManager};
//!
//! let registry = SecretManagerRegistry::new();
//! registry.register("env", Box::new(EnvSecretManager::new()));
//!
//! // Read a secret
//! let api_key = registry.read_secret("env", "OPENAI_API_KEY").await?;
//! ```
//!
//! # Secret Reference Syntax
//!
//! Secrets can be referenced in configuration files using the following syntax:
//! - `${secret:env:SECRET_NAME}` - Read from environment variable
//! - `${secret:file:path/to/secret}` - Read from file
//! - `${secret:aws:secret-name}` - Read from AWS Secrets Manager
//! - `${secret:gcp:secret-name}` - Read from Google Cloud Secret Manager
//! - `${secret:azure:secret-name}` - Read from Azure Key Vault
//! - `${secret:vault:path/to/secret}` - Read from HashiCorp Vault

pub mod env;
pub mod file;
pub mod registry;

// Cloud secret managers
pub mod aws;
pub mod azure;
pub mod gcp;
pub mod vault;

pub use env::EnvSecretManager;
pub use file::FileSecretManager;
pub use registry::SecretManagerRegistry;

// Re-export cloud secret managers
pub use aws::{AwsSecretManager, AwsSecretsConfig};
pub use azure::{AzureSecretManager, AzureSecretsConfig};
pub use gcp::{GcpSecretManager, GcpSecretsConfig};
pub use vault::{VaultConfig, VaultSecretManager};

// Re-export trait types for convenience
pub use crate::core::traits::secret_manager::{
    BoxedSecretManager, ListSecretsOptions, ListSecretsResult, ReadSecretOptions, Secret,
    SecretError, SecretManager, SecretMetadata, SecretResult, WriteSecretOptions,
};
