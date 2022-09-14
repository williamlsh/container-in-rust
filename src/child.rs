use anyhow::{Context, Result};
use std::ffi::CString;

use nix::{
    sched::{clone, CloneFlags},
    sys::signal::Signal,
    unistd::{close, execve, Pid},
};

use crate::{
    capabilities::setcapabilities, config::ContainerOpts, hostname::set_container_hostname,
    mounts::setmountpoint, namespaces::userns, syscalls::set_syscalls,
};

const STACK_SIZE: usize = 1024 * 1024; // 1KiB

fn child(config: ContainerOpts) -> Result<isize> {
    setup_container_configurations(&config)?;
    log::info!(
        "Starting container with command {} and args {:?}",
        config.path.to_str()?,
        config.argv
    );
    match execve::<CString, CString>(&config.path, &config.argv, &[]) {
        Ok(_) => Ok(0),
        Err(_) => Ok(1),
    }
}

pub(crate) fn generate_child_process(config: ContainerOpts) -> Result<Pid> {
    let mut tmp_stack: [u8; STACK_SIZE] = [0; STACK_SIZE];
    let mut flags = CloneFlags::empty();
    flags.insert(CloneFlags::CLONE_NEWNS);
    flags.insert(CloneFlags::CLONE_NEWCGROUP);
    flags.insert(CloneFlags::CLONE_NEWPID);
    flags.insert(CloneFlags::CLONE_NEWIPC);
    flags.insert(CloneFlags::CLONE_NEWNET);
    flags.insert(CloneFlags::CLONE_NEWUTS);
    let child = child(config)?;
    clone(
        Box::new(|| child),
        &mut tmp_stack,
        flags,
        Some(Signal::SIGCHLD as i32),
    )
    .with_context(|| "Could not create child process")
}

fn setup_container_configurations(config: &ContainerOpts) -> Result<()> {
    set_container_hostname(&config.hostname)?;
    setmountpoint(&config.mount_dir, &config.addpaths)?;
    userns(config.fd, config.uid)?;
    close(config.fd).unwrap();
    setcapabilities()?;
    set_syscalls()
}
