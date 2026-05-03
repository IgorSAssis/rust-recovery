use std::fmt;
use std::path::PathBuf;

use crate::constants::BYTES_PER_GIB;

/// Represents a physical storage device available on the system.
#[derive(Debug, Clone, PartialEq)]
pub struct StorageDevice {
    /// Raw device path (e.g. `/dev/sda`, `/dev/nvme0n1`).
    pub path: PathBuf,
    /// Human-readable model name reported by the device firmware.
    pub name: String,
    /// Total capacity in bytes.
    pub size_bytes: u64,
    /// Whether the OS considers the device removable (e.g. USB drive).
    pub removable: bool,
}

impl fmt::Display for StorageDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} — {} ({}, {:.2} GiB)",
            self.path.display(),
            self.name,
            if self.removable { "removable" } else { "fixed" },
            self.size_bytes as f64 / BYTES_PER_GIB as f64,
        )
    }
}
