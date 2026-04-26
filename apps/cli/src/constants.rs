/// Default number of bytes shown by the `hexdump` subcommand when `--length`
/// is not provided. 256 bytes produces 16 rows of 16 bytes each — a compact
/// view suitable for quick forensic inspection at any offset.
pub const DEFAULT_HEXDUMP_LENGTH: usize = 256;
