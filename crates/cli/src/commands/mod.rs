use anyhow::Result;

pub mod scan;

pub trait Command {
    fn run(&self) -> Result<()>;
}
