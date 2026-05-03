use anyhow::Result;
use clap::Args;
use device_detector::DeviceDetector;

use super::Command;

#[derive(Args)]
pub struct DevicesArgs;

#[derive(Default)]
pub struct DevicesCommand;

impl DevicesCommand {
    pub fn new(_args: DevicesArgs) -> Self {
        Self
    }
}

impl Command for DevicesCommand {
    fn run(&mut self) -> Result<()> {
        let devices = DeviceDetector::new().list_devices()?;

        if devices.is_empty() {
            println!("No physical storage devices detected.");
            return Ok(());
        }

        println!("{} device(s) found:\n", devices.len());

        for device in &devices {
            println!("  {device}");
        }

        Ok(())
    }
}
