use super::reader::*;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn create_test_file(data: &[u8]) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push("rust-recover_test.img");

    let mut file = File::create(&path).unwrap();
    file.write_all(data).unwrap();

    path
}

#[test]
fn should_open_existing_file() {
    let data = vec![1, 2, 3, 4];
    let path = create_test_file(&data);
    let reader = DiskReader::open(&path);

    assert!(reader.is_ok());
}

#[test]
fn should_read_bytes_from_file() {
    let data = vec![10, 20, 30, 40];
    let path = create_test_file(&data);
    let mut reader = DiskReader::open(&path).unwrap();
    let mut buffer = [0u8; 4];

    let bytes_read = reader.read_chunk(&mut buffer).unwrap();

    assert_eq!(bytes_read, 4);
    assert_eq!(buffer, [10, 20, 30, 40]);
}

#[test]
fn should_read_bytes_at_offset() {
    let data = vec![0, 1, 2, 3, 4, 5];
    let path = create_test_file(&data);
    let mut reader = DiskReader::open(&path).unwrap();
    let mut buffer = [0u8; 2];

    reader.read_at(2, &mut buffer).unwrap();

    assert_eq!(buffer, [2, 3]);
}

#[test]
fn should_return_error_when_file_not_exists() {
    let result = DiskReader::open("nonexistent_file.img");

    assert!(result.is_err());
}
