use anyhow::Result;

pub mod devices;
pub mod hexdump;
pub mod recover;
pub mod scan;

pub trait Command {
    fn run(&mut self) -> Result<()>;
}
