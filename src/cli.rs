use anyhow::{bail, Result};
use clap::Parser;
use env_logger::Env;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Cli {
    /// Activate debug mode
    #[arg(short, long, action)]
    debug: bool,

    /// Command to execute inside the container.
    #[arg(short, long)]
    pub(crate) command: String,

    /// User ID to create inside the container.
    #[arg(short, long)]
    pub(crate) uid: u32,

    /// Directory to mount as root of the container.
    #[arg(short, long)]
    pub(crate) mount_dir: PathBuf,

    /// Mount a directory inside the container
    #[arg(short, long)]
    pub(crate) addpaths: Vec<PathBuf>,
}

pub fn parse_args() -> Result<Cli> {
    let args = Cli::parse();

    if args.debug {
        setup_log("debug");
    } else {
        setup_log("info");
    }
    if !args
        .mount_dir
        .try_exists()
        .expect("Could not check existence of mount directory")
        || !args.mount_dir.is_dir()
    {
        bail!("Invalid mount_dir");
    }
    if args.command.is_empty() {
        bail!("Empty command");
    }
    Ok(args)
}

fn setup_log(level: &str) {
    env_logger::Builder::from_env(Env::default().default_filter_or(level))
        .format_timestamp_secs()
        .init();
}
