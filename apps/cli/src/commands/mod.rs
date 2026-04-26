use anyhow::Result;

pub mod hexdump;
pub mod recover;
pub mod scan;

pub trait Command {
    fn run(&mut self) -> Result<()>;
}
