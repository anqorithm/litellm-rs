//! Batch processing types and data structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Batch request for processing multiple operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequest {
    /// Unique batch ID
    pub batch_id: String,
    /// User ID who created the batch
    pub user_id: String,
    /// Batch type
    pub batch_type: BatchType,
    /// Individual requests in the batch
    pub requests: Vec<BatchItem>,
    /// Batch metadata
    pub metadata: HashMap<String, String>,
    /// Completion window in hours (24h default)
    pub completion_window: Option<u32>,
    /// Webhook URL for completion notification
    pub webhook_url: Option<String>,
}

/// Types of batch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchType {
    /// Chat completion batch requests
    ChatCompletion,
    /// Embedding batch requests
    Embedding,
    /// Image generation batch requests
    ImageGeneration,
    /// Custom batch request type
    Custom(String),
}

/// Individual item in a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItem {
    /// Custom ID for this request
    pub custom_id: String,
    /// HTTP method (usually POST)
    pub method: String,
    /// API endpoint
    pub url: String,
    /// Request body
    pub body: serde_json::Value,
}

/// Database batch record (different from BatchItem)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRecord {
    /// Batch ID
    pub id: String,
    /// Object type
    pub object: String,
    /// Endpoint used
    pub endpoint: String,
    /// Input file ID
    pub input_file_id: Option<String>,
    /// Completion window
    pub completion_window: String,
    /// Batch status
    pub status: BatchStatus,
    /// Output file ID
    pub output_file_id: Option<String>,
    /// Error file ID
    pub error_file_id: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// In progress timestamp
    pub in_progress_at: Option<DateTime<Utc>>,
    /// Expiration timestamp
    pub expires_at: Option<DateTime<Utc>>,
    /// Finalizing timestamp
    pub finalizing_at: Option<DateTime<Utc>>,
    /// Completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Failed timestamp
    pub failed_at: Option<DateTime<Utc>>,
    /// Expired timestamp
    pub expired_at: Option<DateTime<Utc>>,
    /// Cancelling timestamp
    pub cancelling_at: Option<DateTime<Utc>>,
    /// Cancelled timestamp
    pub cancelled_at: Option<DateTime<Utc>>,
    /// Request counts
    pub request_counts: BatchRequestCounts,
    /// Batch metadata
    pub metadata: Option<serde_json::Value>,
}

/// Batch response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResponse {
    /// Batch ID
    pub id: String,
    /// Object type
    pub object: String,
    /// Endpoint used
    pub endpoint: String,
    /// Batch status
    pub status: BatchStatus,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Expiration timestamp
    pub expires_at: Option<DateTime<Utc>>,
    /// Input file ID (for file-based batches)
    pub input_file_id: Option<String>,
    /// Output file ID (for completed batches)
    pub output_file_id: Option<String>,
    /// Error file ID (for failed requests)
    pub error_file_id: Option<String>,
    /// Request counts
    pub request_counts: BatchRequestCounts,
    /// Batch metadata
    pub metadata: Option<serde_json::Value>,
    /// Completion window
    pub completion_window: String,
    /// In progress timestamp
    pub in_progress_at: Option<DateTime<Utc>>,
    /// Finalizing timestamp
    pub finalizing_at: Option<DateTime<Utc>>,
    /// Failed timestamp
    pub failed_at: Option<DateTime<Utc>>,
    /// Expired timestamp
    pub expired_at: Option<DateTime<Utc>>,
    /// Cancelling timestamp
    pub cancelling_at: Option<DateTime<Utc>>,
    /// Cancelled timestamp
    pub cancelled_at: Option<DateTime<Utc>>,
}

/// Batch processing status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BatchStatus {
    /// Batch is being validated
    Validating,
    /// Batch validation failed
    Failed,
    /// Batch is being processed
    InProgress,
    /// Batch is being finalized
    Finalizing,
    /// Batch processing completed
    Completed,
    /// Batch has expired
    Expired,
    /// Batch is being cancelled
    Cancelling,
    /// Batch has been cancelled
    Cancelled,
}

impl BatchStatus {
    /// Return the canonical status spelling used by APIs and persistence.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Validating => "validating",
            Self::Failed => "failed",
            Self::InProgress => "in_progress",
            Self::Finalizing => "finalizing",
            Self::Completed => "completed",
            Self::Expired => "expired",
            Self::Cancelling => "cancelling",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::str::FromStr for BatchStatus {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "validating" | "Validating" => Ok(Self::Validating),
            "failed" | "Failed" => Ok(Self::Failed),
            "in_progress" | "InProgress" => Ok(Self::InProgress),
            "finalizing" | "Finalizing" => Ok(Self::Finalizing),
            "completed" | "Completed" => Ok(Self::Completed),
            "expired" | "Expired" => Ok(Self::Expired),
            "cancelling" | "Cancelling" => Ok(Self::Cancelling),
            "cancelled" | "Cancelled" => Ok(Self::Cancelled),
            _ => Err("invalid batch status"),
        }
    }
}

/// Request counts for batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequestCounts {
    /// Total requests in batch
    pub total: i32,
    /// Completed requests
    pub completed: i32,
    /// Failed requests
    pub failed: i32,
}

/// Individual batch result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    /// Custom ID from request
    pub custom_id: String,
    /// HTTP response
    pub response: Option<BatchHttpResponse>,
    /// Error information
    pub error: Option<BatchError>,
}

/// HTTP response for batch item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchHttpResponse {
    /// HTTP status code
    pub status_code: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: serde_json::Value,
}

/// Batch error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Additional error details
    pub details: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== BatchRequest Tests ====================

    #[test]
    fn test_batch_request_structure() {
        let request = BatchRequest {
            batch_id: "batch-123".to_string(),
            user_id: "user-456".to_string(),
            batch_type: BatchType::ChatCompletion,
            requests: vec![],
            metadata: HashMap::new(),
            completion_window: Some(24),
            webhook_url: None,
        };

        assert_eq!(request.batch_id, "batch-123");
        assert_eq!(request.user_id, "user-456");
        assert_eq!(request.completion_window, Some(24));
    }

    #[test]
    fn test_batch_request_with_items() {
        let item = BatchItem {
            custom_id: "req-1".to_string(),
            method: "POST".to_string(),
            url: "/v1/chat/completions".to_string(),
            body: serde_json::json!({"model": "gpt-4"}),
        };

        let request = BatchRequest {
            batch_id: "batch-with-items".to_string(),
            user_id: "user-1".to_string(),
            batch_type: BatchType::ChatCompletion,
            requests: vec![item],
            metadata: HashMap::new(),
            completion_window: None,
            webhook_url: Some("https://webhook.example.com".to_string()),
        };

        assert_eq!(request.requests.len(), 1);
        assert!(request.webhook_url.is_some());
    }

    #[test]
    fn test_batch_request_serialization() {
        let request = BatchRequest {
            batch_id: "batch-ser".to_string(),
            user_id: "user-ser".to_string(),
            batch_type: BatchType::Embedding,
            requests: vec![],
            metadata: HashMap::new(),
            completion_window: None,
            webhook_url: None,
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["batch_id"], "batch-ser");
        assert!(json["requests"].is_array());
    }

    // ==================== BatchType Tests ====================

    #[test]
    fn test_batch_type_chat_completion() {
        let batch_type = BatchType::ChatCompletion;
        let json = serde_json::to_string(&batch_type).unwrap();
        assert!(json.contains("ChatCompletion"));
    }

    #[test]
    fn test_batch_type_embedding() {
        let batch_type = BatchType::Embedding;
        let json = serde_json::to_string(&batch_type).unwrap();
        assert!(json.contains("Embedding"));
    }

    #[test]
    fn test_batch_type_custom() {
        let batch_type = BatchType::Custom("custom_type".to_string());
        let json = serde_json::to_string(&batch_type).unwrap();
        assert!(json.contains("custom_type"));
    }

    #[test]
    fn test_batch_type_clone() {
        let batch_type = BatchType::ImageGeneration;
        let cloned = batch_type.clone();
        let json1 = serde_json::to_string(&batch_type).unwrap();
        let json2 = serde_json::to_string(&cloned).unwrap();
        assert_eq!(json1, json2);
    }

    // ==================== BatchItem Tests ====================

    #[test]
    fn test_batch_item_structure() {
        let item = BatchItem {
            custom_id: "item-123".to_string(),
            method: "POST".to_string(),
            url: "/v1/chat/completions".to_string(),
            body: serde_json::json!({"model": "gpt-4", "messages": []}),
        };

        assert_eq!(item.custom_id, "item-123");
        assert_eq!(item.method, "POST");
        assert!(item.url.contains("/chat/completions"));
    }

    #[test]
    fn test_batch_item_serialization() {
        let item = BatchItem {
            custom_id: "ser-item".to_string(),
            method: "POST".to_string(),
            url: "/v1/embeddings".to_string(),
            body: serde_json::json!({"input": "hello"}),
        };

        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["custom_id"], "ser-item");
        assert_eq!(json["method"], "POST");
    }

    #[test]
    fn test_batch_item_deserialization() {
        let json = r#"{
            "custom_id": "deser-item",
            "method": "POST",
            "url": "/v1/chat/completions",
            "body": {"model": "gpt-4"}
        }"#;

        let item: BatchItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.custom_id, "deser-item");
        assert_eq!(item.method, "POST");
    }

    // ==================== BatchStatus Tests ====================

    #[test]
    fn test_batch_status_variants() {
        let statuses = [
            (BatchStatus::Validating, "validating", "Validating"),
            (BatchStatus::Failed, "failed", "Failed"),
            (BatchStatus::InProgress, "in_progress", "InProgress"),
            (BatchStatus::Finalizing, "finalizing", "Finalizing"),
            (BatchStatus::Completed, "completed", "Completed"),
            (BatchStatus::Expired, "expired", "Expired"),
            (BatchStatus::Cancelling, "cancelling", "Cancelling"),
            (BatchStatus::Cancelled, "cancelled", "Cancelled"),
        ];

        for (status, canonical, historical) in statuses {
            assert_eq!(status.as_str(), canonical);
            assert_eq!(canonical.parse::<BatchStatus>().unwrap(), status);
            assert_eq!(historical.parse::<BatchStatus>().unwrap(), status);
            assert_eq!(
                serde_json::to_string(&status).unwrap(),
                format!("\"{canonical}\"")
            );
            assert_eq!(
                serde_json::from_str::<BatchStatus>(&format!("\"{canonical}\"")).unwrap(),
                status
            );
        }

        assert!("unknown".parse::<BatchStatus>().is_err());
        assert!(serde_json::from_str::<BatchStatus>("\"InProgress\"").is_err());
    }

    #[test]
    fn test_batch_status_equality() {
        assert_eq!(BatchStatus::Completed, BatchStatus::Completed);
        assert_ne!(BatchStatus::Completed, BatchStatus::Failed);
    }

    #[test]
    fn test_batch_status_serialization() {
        let status = BatchStatus::InProgress;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"in_progress\"");
    }

    // ==================== BatchRequestCounts Tests ====================

    #[test]
    fn test_batch_request_counts_structure() {
        let counts = BatchRequestCounts {
            total: 100,
            completed: 50,
            failed: 5,
        };

        assert_eq!(counts.total, 100);
        assert_eq!(counts.completed, 50);
        assert_eq!(counts.failed, 5);
    }

    #[test]
    fn test_batch_request_counts_zero() {
        let counts = BatchRequestCounts {
            total: 0,
            completed: 0,
            failed: 0,
        };

        assert_eq!(counts.total, 0);
    }

    #[test]
    fn test_batch_request_counts_serialization() {
        let counts = BatchRequestCounts {
            total: 10,
            completed: 8,
            failed: 2,
        };

        let json = serde_json::to_value(&counts).unwrap();
        assert_eq!(json["total"], 10);
        assert_eq!(json["completed"], 8);
        assert_eq!(json["failed"], 2);
    }

    // ==================== BatchResult Tests ====================

    #[test]
    fn test_batch_result_success() {
        let response = BatchHttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: serde_json::json!({"id": "msg-123"}),
        };

        let result = BatchResult {
            custom_id: "success-req".to_string(),
            response: Some(response),
            error: None,
        };

        assert!(result.response.is_some());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_batch_result_failure() {
        let error = BatchError {
            code: "rate_limit_exceeded".to_string(),
            message: "Rate limit exceeded".to_string(),
            details: None,
        };

        let result = BatchResult {
            custom_id: "failed-req".to_string(),
            response: None,
            error: Some(error),
        };

        assert!(result.response.is_none());
        assert!(result.error.is_some());
    }

    // ==================== BatchHttpResponse Tests ====================

    #[test]
    fn test_batch_http_response_structure() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let response = BatchHttpResponse {
            status_code: 200,
            headers,
            body: serde_json::json!({"result": "success"}),
        };

        assert_eq!(response.status_code, 200);
        assert!(response.headers.contains_key("Content-Type"));
    }

    #[test]
    fn test_batch_http_response_error_status() {
        let response = BatchHttpResponse {
            status_code: 429,
            headers: HashMap::new(),
            body: serde_json::json!({"error": "rate limited"}),
        };

        assert_eq!(response.status_code, 429);
    }

    // ==================== BatchError Tests ====================

    #[test]
    fn test_batch_error_structure() {
        let error = BatchError {
            code: "invalid_request".to_string(),
            message: "Invalid request format".to_string(),
            details: Some(serde_json::json!({"field": "model"})),
        };

        assert_eq!(error.code, "invalid_request");
        assert!(error.details.is_some());
    }

    #[test]
    fn test_batch_error_no_details() {
        let error = BatchError {
            code: "server_error".to_string(),
            message: "Internal server error".to_string(),
            details: None,
        };

        assert!(error.details.is_none());
    }

    #[test]
    fn test_batch_error_serialization() {
        let error = BatchError {
            code: "ser_error".to_string(),
            message: "Serialization test".to_string(),
            details: None,
        };

        let json = serde_json::to_value(&error).unwrap();
        assert_eq!(json["code"], "ser_error");
        assert_eq!(json["message"], "Serialization test");
    }

    // ==================== Clone Tests ====================

    #[test]
    fn test_batch_item_clone() {
        let item = BatchItem {
            custom_id: "clone-test".to_string(),
            method: "POST".to_string(),
            url: "/test".to_string(),
            body: serde_json::json!({}),
        };

        let cloned = item.clone();
        assert_eq!(item.custom_id, cloned.custom_id);
    }

    #[test]
    fn test_batch_result_clone() {
        let result = BatchResult {
            custom_id: "clone-result".to_string(),
            response: None,
            error: None,
        };

        let cloned = result.clone();
        assert_eq!(result.custom_id, cloned.custom_id);
    }
}
