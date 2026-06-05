use std::path::Path;

use anyhow::{Result, bail};

pub struct SourceValidator;

impl SourceValidator {
    /// Validates that `source` is a readable regular file or block device.
    ///
    /// Returns an error if:
    /// - The path does not exist
    /// - The path is a directory or other unsupported file type
    ///
    /// Block device detection is only performed on Unix systems.
    pub fn validate(source: &Path) -> Result<()> {
        let metadata = source
            .metadata()
            .map_err(|e| anyhow::anyhow!("Cannot access '{}': {}", source.display(), e))?;

        let file_type = metadata.file_type();

        if file_type.is_dir() {
            bail!(
                "'{}' is a directory. Provide a disk image file or block device path.",
                source.display()
            );
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;

            if !file_type.is_file() && !file_type.is_block_device() {
                bail!(
                    "'{}' is not a regular file or block device.",
                    source.display()
                );
            }
        }

        #[cfg(not(unix))]
        {
            if !file_type.is_file() {
                bail!("'{}' is not a regular file.", source.display());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "validation_tests.rs"]
mod validation_tests;
