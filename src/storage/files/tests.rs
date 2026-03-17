//! Tests for file storage implementations

use super::default_data_path;
use super::local::LocalStorage;
use tempfile::TempDir;

#[tokio::test]
async fn test_local_storage() {
    let temp_dir = TempDir::new().unwrap();
    let storage = LocalStorage::new(temp_dir.path().to_str().unwrap())
        .await
        .unwrap();

    // Test store
    let content = b"Hello, World!";
    let file_id = storage.store("test.txt", content).await.unwrap();
    assert!(!file_id.is_empty());

    // Test exists
    assert!(storage.exists(&file_id).await.unwrap());

    // Test get
    let retrieved = storage.get(&file_id).await.unwrap();
    assert_eq!(retrieved, content);

    // Test metadata
    let metadata = storage.metadata(&file_id).await.unwrap();
    assert_eq!(metadata.filename, "test.txt");
    assert_eq!(metadata.size, content.len() as u64);

    // Test delete
    storage.delete(&file_id).await.unwrap();
    assert!(!storage.exists(&file_id).await.unwrap());
}

#[test]
fn test_default_data_path_is_absolute() {
    // Ensure the default path is absolute, not relative like the old "./data"
    let path = default_data_path();
    assert!(
        path.is_absolute(),
        "default_data_path() should return an absolute path, got: {}",
        path.display()
    );
}

#[test]
fn test_default_data_path_ends_with_data() {
    let path = default_data_path();
    assert!(
        path.ends_with("litellm-rs/data"),
        "default_data_path() should end with litellm-rs/data, got: {}",
        path.display()
    );
}

#[test]
fn test_default_data_path_env_override() {
    let original = std::env::var("LITELLM_DATA_DIR").ok();
    // SAFETY: This test is not run in parallel with other tests that read
    // LITELLM_DATA_DIR. set_var/remove_var are unsafe because they are not
    // thread-safe, but #[test] functions run serially by default.
    unsafe {
        std::env::set_var("LITELLM_DATA_DIR", "/custom/storage/path");
    }
    let path = default_data_path();
    assert_eq!(
        path,
        std::path::PathBuf::from("/custom/storage/path"),
        "LITELLM_DATA_DIR should override the default path"
    );
    // Restore original env
    unsafe {
        match original {
            Some(val) => std::env::set_var("LITELLM_DATA_DIR", val),
            None => std::env::remove_var("LITELLM_DATA_DIR"),
        }
    }
}

#[test]
fn test_content_type_detection() {
    assert_eq!(LocalStorage::detect_content_type("test.txt"), "text/plain");
    assert_eq!(
        LocalStorage::detect_content_type("data.json"),
        "application/json"
    );
    assert_eq!(LocalStorage::detect_content_type("image.png"), "image/png");
    assert_eq!(
        LocalStorage::detect_content_type("unknown"),
        "application/octet-stream"
    );
}
