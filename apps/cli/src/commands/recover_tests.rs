use std::path::PathBuf;

use file_carver::constants::DEFAULT_CHUNK_SIZE;
use file_carver::signature::{FileKind, SUPPORTED_SIGNATURES};

use super::{RecoverArgs, RecoverCommand, RecoveryStrategyKind};

fn make_command(types: Option<Vec<FileKind>>) -> RecoverCommand {
    RecoverCommand::new(RecoverArgs {
        source: PathBuf::from("dummy.img"),
        output: PathBuf::from("/tmp/out"),
        types,
        chunk_size: DEFAULT_CHUNK_SIZE,
        strategy: RecoveryStrategyKind::Carver,
    })
}

#[test]
fn resolve_signatures_returns_all_when_types_is_none() {
    let cmd = make_command(None);
    let result = cmd.resolve_signatures();
    assert_eq!(result.len(), SUPPORTED_SIGNATURES.len());
}

#[test]
fn resolve_signatures_returns_only_jpeg_when_types_is_jpeg() {
    let cmd = make_command(Some(vec![FileKind::Jpeg]));
    let result = cmd.resolve_signatures();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].kind, FileKind::Jpeg);
}

#[test]
fn resolve_signatures_returns_jpeg_and_png_when_types_contains_both() {
    let cmd = make_command(Some(vec![FileKind::Jpeg, FileKind::Png]));
    let result = cmd.resolve_signatures();
    assert_eq!(result.len(), 2);
    assert!(result.iter().any(|s| s.kind == FileKind::Jpeg));
    assert!(result.iter().any(|s| s.kind == FileKind::Png));
}

#[test]
fn resolve_signatures_returns_empty_when_types_is_empty_vec() {
    let cmd = make_command(Some(vec![]));
    let result = cmd.resolve_signatures();
    assert!(result.is_empty());
}
