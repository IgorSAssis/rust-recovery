use anyhow::Result;

pub mod recover;
pub mod scan;

pub trait Command {
    fn run(&mut self) -> Result<()>;
}
