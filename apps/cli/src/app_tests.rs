use clap::Parser;
use file_carver::signature::FileKind;

use super::{Cli, Commands};

#[test]
fn scan_subcommand_parses_source_arg() {
    let cli = Cli::try_parse_from(["rustrecovery", "scan", "--source", "disk.img"]).unwrap();
    let Commands::Scan(args) = cli.command else {
        panic!("expected Scan variant");
    };
    assert_eq!(args.source.to_str().unwrap(), "disk.img");
}

#[test]
fn recover_subcommand_parses_source_and_output() {
    let cli = Cli::try_parse_from([
        "rustrecovery",
        "recover",
        "--source",
        "disk.img",
        "--output",
        "out/",
    ])
    .unwrap();
    let Commands::Recover(args) = cli.command else {
        panic!("expected Recover variant");
    };
    assert_eq!(args.source.to_str().unwrap(), "disk.img");
    assert_eq!(args.output.to_str().unwrap(), "out/");
}

#[test]
fn recover_types_arg_parses_jpg_and_png() {
    let cli = Cli::try_parse_from([
        "rustrecovery",
        "recover",
        "--source",
        "disk.img",
        "--output",
        "out/",
        "--types",
        "jpg,png",
    ])
    .unwrap();
    let Commands::Recover(args) = cli.command else {
        panic!("expected Recover variant");
    };
    let types = args.types.unwrap();
    assert_eq!(types.len(), 2);
    assert!(types.contains(&FileKind::Jpeg));
    assert!(types.contains(&FileKind::Png));
}

#[test]
fn recover_types_arg_parses_pdf() {
    let cli = Cli::try_parse_from([
        "rustrecovery",
        "recover",
        "--source",
        "disk.img",
        "--output",
        "out/",
        "--types",
        "pdf",
    ])
    .unwrap();
    let Commands::Recover(args) = cli.command else {
        panic!("expected Recover variant");
    };
    let types = args.types.unwrap();
    assert_eq!(types.len(), 1);
    assert!(types.contains(&FileKind::Pdf));
}

#[test]
fn scan_without_source_returns_error() {
    let result = Cli::try_parse_from(["rustrecovery", "scan"]);
    assert!(result.is_err());
}

#[test]
fn recover_without_output_returns_error() {
    let result =
        Cli::try_parse_from(["rustrecovery", "recover", "--source", "disk.img"]);
    assert!(result.is_err());
}

#[test]
fn recover_chunk_size_arg_is_parsed() {
    let cli = Cli::try_parse_from([
        "rustrecovery",
        "recover",
        "--source",
        "disk.img",
        "--output",
        "out/",
        "--chunk-size",
        "4096",
    ])
    .unwrap();
    let Commands::Recover(args) = cli.command else {
        panic!("expected Recover variant");
    };
    assert_eq!(args.chunk_size, 4096);
}
