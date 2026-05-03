use std::fs;
use std::path::Path;

use crate::constants::{BYTES_PER_BLOCK, DEV_PATH_PREFIX, SYS_BLOCK_PATH};
use crate::device::StorageDevice;
use crate::error::DeviceDetectorError;

#[derive(Default)]
pub struct LinuxDeviceDetector;

impl LinuxDeviceDetector {
    pub fn new() -> Self {
        Self
    }

    /// Lists all physical block devices detected on the system.
    ///
    /// Reads from `/sys/block/` and filters virtual devices (e.g. zram, loop).
    /// Only whole disks are returned — partitions are excluded because
    /// `/sys/block/` only exposes top-level block devices.
    pub fn list_devices(&self) -> Result<Vec<StorageDevice>, DeviceDetectorError> {
        let entries = fs::read_dir(SYS_BLOCK_PATH).map_err(|e| DeviceDetectorError::Io {
            path: SYS_BLOCK_PATH.to_string(),
            source: e,
        })?;

        let mut devices = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| DeviceDetectorError::Io {
                path: SYS_BLOCK_PATH.to_string(),
                source: e,
            })?;

            let sys_path = entry.path();

            if !Self::is_physical_device(&sys_path) {
                continue;
            }

            let dev_name = entry.file_name().to_string_lossy().into_owned();

            let size_bytes = Self::parse_size_bytes(&Self::read_sysfs_field(
                &sys_path.join("size"),
            )?)?;

            let removable =
                Self::parse_removable(&Self::read_sysfs_field(&sys_path.join("removable"))?);

            let model_path = sys_path.join("device").join("model");
            let name = if model_path.exists() {
                Self::parse_model(&Self::read_sysfs_field(&model_path)?)
            } else {
                dev_name.clone()
            };

            devices.push(StorageDevice {
                path: Path::new(DEV_PATH_PREFIX).join(&dev_name),
                name,
                size_bytes,
                removable,
            });
        }

        devices.sort_by(|a, b| a.path.cmp(&b.path));

        Ok(devices)
    }

    /// Returns `true` if the sysfs path represents a physical device.
    ///
    /// Virtual devices (e.g. `zram0`, loop) lack the `device/` subdirectory.
    fn is_physical_device(sys_path: &Path) -> bool {
        sys_path.join("device").is_dir()
    }

    fn read_sysfs_field(path: &Path) -> Result<String, DeviceDetectorError> {
        fs::read_to_string(path).map_err(|e| DeviceDetectorError::Io {
            path: path.display().to_string(),
            source: e,
        })
    }

    /// Converts the 512-byte block count from `/sys/block/<dev>/size` to bytes.
    pub(crate) fn parse_size_bytes(content: &str) -> Result<u64, DeviceDetectorError> {
        let blocks: u64 = content
            .trim()
            .parse()
            .map_err(|_| DeviceDetectorError::Parse {
                field: "size",
                raw_value: content.trim().to_string(),
            })?;

        Ok(blocks * BYTES_PER_BLOCK)
    }

    /// Parses `/sys/block/<dev>/removable`: `"1"` → `true`, anything else → `false`.
    pub(crate) fn parse_removable(content: &str) -> bool {
        content.trim() == "1"
    }

    /// Trims kernel-padded whitespace from a device model name.
    pub(crate) fn parse_model(content: &str) -> String {
        content.trim().to_string()
    }
}

#[cfg(test)]
#[path = "linux_tests.rs"]
mod linux_tests;
