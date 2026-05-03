mod constants;
pub mod device;
pub mod error;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

pub use device::StorageDevice;
pub use error::DeviceDetectorError;

/// Entry point for device enumeration.
///
/// Delegates to the platform-specific implementation at compile time.
/// Use [`DeviceDetector::new`] or [`DeviceDetector::default`] to obtain
/// an instance, then call [`list_devices`][DeviceDetector::list_devices].
#[derive(Default)]
pub struct DeviceDetector;

impl DeviceDetector {
    pub fn new() -> Self {
        Self
    }

    /// Lists all physical storage devices available on the current system.
    ///
    /// On Linux, reads from `/sys/block/` filtering out virtual devices.
    /// On Windows, delegates to the Windows device enumeration API (stub).
    pub fn list_devices(&self) -> Result<Vec<StorageDevice>, DeviceDetectorError> {
        #[cfg(target_os = "linux")]
        {
            linux::LinuxDeviceDetector::new().list_devices()
        }

        #[cfg(target_os = "windows")]
        {
            windows::WindowsDeviceDetector::new().list_devices()
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            Ok(Vec::new())
        }
    }
}
