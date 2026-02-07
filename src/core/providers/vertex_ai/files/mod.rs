//! Vertex AI Files Module
//!
//! Handles file uploads, management, and processing for Vertex AI

use crate::ProviderError;
use std::sync::LazyLock;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Regex for matching GCS URIs
static GCS_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"gs://[^\s]+").unwrap());

/// Regex for matching file IDs
static FILE_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"files/[a-zA-Z0-9\-_]+").unwrap());

/// File upload request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUploadRequest {
    pub display_name: String,
    pub mime_type: String,
    pub data: FileData,
}

/// File data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FileData {
    /// Base64 encoded content
    Base64 { content: String },
    /// Binary content
    Binary {
        #[serde(skip_serializing)]
        content: Vec<u8>,
    },
    /// File path for local files
    FilePath {
        #[serde(skip_serializing)]
        path: String,
    },
}

/// File metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub name: String,
    pub display_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub create_time: String,
    pub update_time: String,
    pub expiration_time: Option<String>,
    pub sha256_hash: String,
    pub uri: String,
    pub state: FileState,
    pub error: Option<FileError>,
    pub video_metadata: Option<VideoMetadata>,
}

/// File state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileState {
    #[serde(rename = "STATE_UNSPECIFIED")]
    Unspecified,
    #[serde(rename = "PROCESSING")]
    Processing,
    #[serde(rename = "ACTIVE")]
    Active,
    #[serde(rename = "FAILED")]
    Failed,
}

/// File error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileError {
    pub code: i32,
    pub message: String,
}

/// Video metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub video_duration: String, // Duration in seconds with up to nine fractional digits
}

/// File list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFilesResponse {
    pub files: Vec<FileMetadata>,
    pub next_page_token: Option<String>,
}

/// File handler for managing uploads and operations
pub struct FileHandler {
    project_id: String,
    location: String,
}

impl FileHandler {
    /// Create new file handler
    pub fn new(project_id: String, location: String) -> Self {
        Self {
            project_id,
            location,
        }
    }

    /// Upload a file to Vertex AI
    pub async fn upload_file(
        &self,
        request: FileUploadRequest,
    ) -> Result<FileMetadata, ProviderError> {
        // Validate file
        self.validate_file_upload(&request)?;

        // TODO: Implement actual file upload via Vertex AI API
        // For now, return a mock response
        Ok(FileMetadata {
            name: format!("files/{}", uuid::Uuid::new_v4()),
            display_name: request.display_name,
            mime_type: request.mime_type,
            size_bytes: self.calculate_file_size(&request.data),
            create_time: chrono::Utc::now().to_rfc3339(),
            update_time: chrono::Utc::now().to_rfc3339(),
            expiration_time: Some((chrono::Utc::now() + chrono::Duration::hours(24)).to_rfc3339()),
            sha256_hash: "mock_hash".to_string(),
            uri: format!(
                "https://storage.googleapis.com/vertex-ai-files/{}",
                uuid::Uuid::new_v4()
            ),
            state: FileState::Processing,
            error: None,
            video_metadata: None,
        })
    }

    /// Get file metadata
    pub async fn get_file(&self, _file_id: &str) -> Result<FileMetadata, ProviderError> {
        // TODO: Implement actual file retrieval
        Err(ProviderError::not_supported(
            "vertex_ai",
            "File retrieval not yet implemented",
        ))
    }

    /// List files
    pub async fn list_files(
        &self,
        _page_size: Option<i32>,
        _page_token: Option<String>,
    ) -> Result<ListFilesResponse, ProviderError> {
        // TODO: Implement actual file listing
        Ok(ListFilesResponse {
            files: Vec::new(),
            next_page_token: None,
        })
    }

    /// Delete a file
    pub async fn delete_file(&self, _file_id: &str) -> Result<(), ProviderError> {
        // TODO: Implement actual file deletion
        Ok(())
    }

    /// Validate file upload request
    fn validate_file_upload(&self, request: &FileUploadRequest) -> Result<(), ProviderError> {
        // Check MIME type
        if !self.is_supported_mime_type(&request.mime_type) {
            return Err(ProviderError::invalid_request(
                "vertex_ai",
                format!("Unsupported MIME type: {}", request.mime_type),
            ));
        }

        // Check file size
        let size = self.calculate_file_size(&request.data);
        if size > self.max_file_size(&request.mime_type) {
            return Err(ProviderError::invalid_request(
                "vertex_ai",
                format!("File size {} bytes exceeds maximum allowed", size),
            ));
        }

        Ok(())
    }

    /// Check if MIME type is supported
    fn is_supported_mime_type(&self, mime_type: &str) -> bool {
        matches!(
            mime_type,
            // Images
            "image/jpeg" | "image/png" | "image/webp" | "image/heic" | "image/heif" |
            // Videos
            "video/mp4" | "video/mpeg" | "video/mov" | "video/avi" | "video/x-flv" | 
            "video/mpg" | "video/webm" | "video/wmv" | "video/3gpp" |
            // Audio
            "audio/wav" | "audio/mp3" | "audio/aiff" | "audio/aac" | "audio/ogg" | "audio/flac" |
            // Documents
            "application/pdf" | "text/plain" | "text/csv" | "text/html" |
            "application/rtf" | "application/epub+zip"
        )
    }

    /// Get maximum file size for MIME type
    fn max_file_size(&self, mime_type: &str) -> i64 {
        match mime_type {
            mime if mime.starts_with("video/") => 2_000_000_000, // 2GB for videos
            mime if mime.starts_with("audio/") => 500_000_000,   // 500MB for audio
            mime if mime.starts_with("image/") => 20_000_000,    // 20MB for images
            _ => 50_000_000,                                     // 50MB for documents
        }
    }

    /// Calculate file size
    fn calculate_file_size(&self, data: &FileData) -> i64 {
        match data {
            FileData::Base64 { content } => {
                // Base64 encoding adds ~33% overhead
                ((content.len() * 3) / 4) as i64
            }
            FileData::Binary { content } => content.len() as i64,
            FileData::FilePath { .. } => 0, // Would need to read file
        }
    }

    /// Convert file to Vertex AI format for multimodal requests
    pub fn to_vertex_format(&self, file_metadata: &FileMetadata) -> Value {
        serde_json::json!({
            "fileData": {
                "mimeType": file_metadata.mime_type,
                "fileUri": file_metadata.uri
            }
        })
    }

    /// Create file reference for use in requests
    pub fn create_file_reference(&self, file_metadata: &FileMetadata) -> FileReference {
        FileReference {
            name: file_metadata.name.clone(),
            mime_type: file_metadata.mime_type.clone(),
            uri: file_metadata.uri.clone(),
        }
    }
}

/// File reference for use in requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReference {
    pub name: String,
    pub mime_type: String,
    pub uri: String,
}

/// File transformation utilities
pub struct FileTransformation;

impl FileTransformation {
    /// Transform file for Gemini chat request
    pub fn transform_for_chat(file_ref: &FileReference) -> Value {
        serde_json::json!({
            "fileData": {
                "mimeType": file_ref.mime_type,
                "fileUri": file_ref.uri
            }
        })
    }

    /// Extract file references from text
    pub fn extract_file_references(text: &str) -> Vec<String> {
        // Extract GCS URIs and file IDs from text
        let mut references = Vec::new();

        // Look for gs:// URIs
        for mat in GCS_PATTERN.find_iter(text) {
            references.push(mat.as_str().to_string());
        }

        // Look for files/* patterns
        for mat in FILE_PATTERN.find_iter(text) {
            references.push(mat.as_str().to_string());
        }

        references
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_supported_mime_type() {
        let handler = FileHandler::new("test".to_string(), "us-central1".to_string());

        assert!(handler.is_supported_mime_type("image/jpeg"));
        assert!(handler.is_supported_mime_type("video/mp4"));
        assert!(handler.is_supported_mime_type("audio/wav"));
        assert!(handler.is_supported_mime_type("application/pdf"));
        assert!(!handler.is_supported_mime_type("application/zip"));
    }

    #[test]
    fn test_calculate_file_size() {
        let handler = FileHandler::new("test".to_string(), "us-central1".to_string());

        let base64_data = FileData::Base64 {
            content: "SGVsbG8gd29ybGQ=".to_string(), // "Hello world" in base64
        };
        assert!(handler.calculate_file_size(&base64_data) > 0);

        let binary_data = FileData::Binary {
            content: vec![1, 2, 3, 4, 5],
        };
        assert_eq!(handler.calculate_file_size(&binary_data), 5);
    }

    #[test]
    fn test_extract_file_references() {
        let text = "Check out this file: gs://my-bucket/video.mp4 and also files/abc123def";
        let refs = FileTransformation::extract_file_references(text);

        assert_eq!(refs.len(), 2);
        assert!(refs.contains(&"gs://my-bucket/video.mp4".to_string()));
        assert!(refs.contains(&"files/abc123def".to_string()));
    }
}
