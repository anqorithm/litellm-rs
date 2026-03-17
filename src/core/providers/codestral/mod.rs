//! Codestral Provider
//!
//! Mistral's specialized code generation model API.
//! Supports fill-in-the-middle (FIM) completion for code.

mod config;
mod error;
mod model_info;
mod provider;

pub use config::CodestralConfig;
pub use error::CodestralError;
pub use model_info::get_model_info;
pub use provider::CodestralProvider;
