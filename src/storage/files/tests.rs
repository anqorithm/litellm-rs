//! Tests for file storage implementations

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
