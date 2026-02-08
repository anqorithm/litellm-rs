//! API validation utilities

use crate::utils::error::error::{GatewayError, Result};
use uuid::Uuid;

/// API validation utilities
pub struct ApiValidator;

impl ApiValidator {
    /// Validate API key format
    pub fn validate_api_key(api_key: &str) -> Result<()> {
        if api_key.trim().is_empty() {
            return Err(GatewayError::Validation(
                "API key cannot be empty".to_string(),
            ));
        }

        if api_key.len() < 10 {
            return Err(GatewayError::Validation("API key is too short".to_string()));
        }

        if api_key.len() > 200 {
            return Err(GatewayError::Validation("API key is too long".to_string()));
        }

        Ok(())
    }

    /// Validate UUID format
    pub fn validate_uuid(uuid_str: &str) -> Result<Uuid> {
        Uuid::parse_str(uuid_str)
            .map_err(|e| GatewayError::Validation(format!("Invalid UUID format: {}", e)))
    }

    /// Validate pagination parameters
    pub fn validate_pagination(page: Option<u32>, limit: Option<u32>) -> Result<(u32, u32)> {
        let page = page.unwrap_or(1);
        let limit = limit.unwrap_or(20);

        if page == 0 {
            return Err(GatewayError::Validation(
                "Page must be greater than 0".to_string(),
            ));
        }

        if limit == 0 {
            return Err(GatewayError::Validation(
                "Limit must be greater than 0".to_string(),
            ));
        }

        if limit > 1000 {
            return Err(GatewayError::Validation(
                "Limit cannot exceed 1000".to_string(),
            ));
        }

        Ok((page, limit))
    }

    /// Validate date range
    pub fn validate_date_range(
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<()> {
        if let (Some(start), Some(end)) = (start_date, end_date) {
            if start >= end {
                return Err(GatewayError::Validation(
                    "Start date must be before end date".to_string(),
                ));
            }

            let max_range = chrono::Duration::days(365);
            if end - start > max_range {
                return Err(GatewayError::Validation(
                    "Date range cannot exceed 365 days".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Validate sort parameters
    pub fn validate_sort_params(
        sort_by: &str,
        sort_order: &str,
        valid_fields: &[&str],
    ) -> Result<()> {
        if !valid_fields.contains(&sort_by) {
            return Err(GatewayError::Validation(format!(
                "Invalid sort field: {}. Valid fields: {:?}",
                sort_by, valid_fields
            )));
        }

        if !["asc", "desc"].contains(&sort_order) {
            return Err(GatewayError::Validation(
                "Sort order must be 'asc' or 'desc'".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate filter parameters
    pub fn validate_filters(
        filters: &std::collections::HashMap<String, String>,
        valid_filters: &[&str],
    ) -> Result<()> {
        for key in filters.keys() {
            if !valid_filters.contains(&key.as_str()) {
                return Err(GatewayError::Validation(format!(
                    "Invalid filter: {}. Valid filters: {:?}",
                    key, valid_filters
                )));
            }
        }

        Ok(())
    }
}
