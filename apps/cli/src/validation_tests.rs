use std::fs;
use std::path::Path;

use super::SourceValidator;

#[test]
fn validate_returns_ok_for_existing_regular_file() {
    let file = tempfile("validate_ok.img");
    fs::write(&file, b"data").unwrap();

    assert!(SourceValidator::validate(&file).is_ok());

    fs::remove_file(&file).unwrap();
}

#[test]
fn validate_returns_error_for_nonexistent_path() {
    let result = SourceValidator::validate(Path::new("/nonexistent/path/disk.img"));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Cannot access"));
}

#[test]
fn validate_returns_error_for_directory() {
    let dir = tempfile("validate_dir");
    fs::create_dir_all(&dir).unwrap();

    let result = SourceValidator::validate(&dir);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("directory"));

    fs::remove_dir_all(&dir).unwrap();
}

fn tempfile(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("rustrecovery_validation_test_{label}"))
}
