use crate::{
    child::generate_child_process,
    cli::Cli,
    config::ContainerOpts,
    mounts::clean_mounts,
    namespaces::handle_child_uid_map,
    resources::{clean_cgroups, restrict_resources},
};
use anyhow::{bail, Context, Result};
use log::{debug, warn};
use nix::{
    sys::{utsname::uname, wait::waitpid},
    unistd::{close, Pid},
};
use scan_fmt::scan_fmt;
use std::{os::unix::prelude::RawFd, path::PathBuf};

pub const MINIMAL_KERNEL_VERSION: f32 = 4.8;

pub(crate) struct Container {
    sockets: (RawFd, RawFd),
    config: ContainerOpts,
    child_pid: Option<Pid>,
}

impl Container {
    pub(crate) fn new(args: Cli) -> Result<Container> {
        let mut addpaths = vec![];
        for ap_pair in args.addpaths.iter() {
            let mut pair = ap_pair.to_str().unwrap().split(":");
            let frompath = PathBuf::from(pair.next().unwrap())
                .canonicalize()
                .expect("Cannot canonicalize path")
                .to_path_buf();
            let mntpath = PathBuf::from(pair.next().unwrap())
                .strip_prefix("/")
                .expect("Cannot strip prefix from path")
                .to_path_buf();
            addpaths.push((frompath, mntpath));
        }
        let (config, sockets) =
            ContainerOpts::new(args.command, args.uid, args.mount_dir, addpaths)?;
        Ok(Container {
            sockets,
            config,
            child_pid: None,
        })
    }

    pub(crate) fn create(&mut self) -> Result<()> {
        let pid = generate_child_process(self.config.clone())
            .with_context(|| "Couldn't generate child process")?;
        restrict_resources(&self.config.hostname, pid);
        handle_child_uid_map(pid, self.sockets.0);
        self.child_pid = Some(pid);
        log::debug!("Container created.");
        Ok(())
    }

    pub(crate) fn clean_exit(&mut self) -> Result<(), String> {
        log::debug!("Cleaning container.");
        close(self.sockets.0).unwrap();
        close(self.sockets.1).unwrap();
        clean_mounts(&self.config.mount_dir).unwrap();
        clean_cgroups(&self.config.hostname);
        Ok(())
    }
}

impl Drop for Container {
    fn drop(&mut self) {
        self.clean_exit().expect("exit");
    }
}

pub fn start(args: Cli) -> Result<()> {
    check_linux_version()?;

    let mut container = Container::new(args)?;
    log::debug!(
        "Container sockets: ({}, {})",
        container.sockets.0,
        container.sockets.1
    );

    container.create()?;
    log::debug!("Container child pid: {:?}", container.child_pid);

    wait_child(container.child_pid).unwrap();

    Ok(())
}

pub fn check_linux_version() -> Result<()> {
    let host = uname()?;
    debug!("Linux release: {:?}", host.release());

    if host.machine() != "x86_64" {
        bail!("Not supported");
    }

    let version = scan_fmt!(host.release().to_str().unwrap(), "{f}.{}", f32)
        .with_context(|| "Failed to get current kernel version")?;
    if version < MINIMAL_KERNEL_VERSION {
        warn!(
            "Current kernel version {} is less than minimum version {}",
            version, MINIMAL_KERNEL_VERSION
        );
        bail!("Low Linux kernel version");
    }

    Ok(())
}

fn wait_child(pid: Option<Pid>) -> nix::Result<()> {
    if let Some(child_pid) = pid {
        log::debug!("Waiting for child (pid {}) to finish", child_pid);
        return waitpid(child_pid, None).map(|_| ());
    }
    Ok(())
}
