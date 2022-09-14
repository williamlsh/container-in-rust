use anyhow::Result;
use container_in_rust::{cli, container};

fn main() -> Result<()> {
    env_logger::init();

    let args = cli::parse_args()?;
    container::start(args)
}
