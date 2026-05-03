use crate::device::StorageDevice;
use crate::error::DeviceDetectorError;

#[derive(Default)]
pub struct WindowsDeviceDetector;

impl WindowsDeviceDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn list_devices(&self) -> Result<Vec<StorageDevice>, DeviceDetectorError> {
        // TODO: implement via winapi / windows crate (SetupDiGetClassDevs)
        Ok(Vec::new())
    }
}
