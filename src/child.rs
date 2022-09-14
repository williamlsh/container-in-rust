use std::ffi::CString;

use nix::{
    sched::{clone, CloneFlags},
    sys::signal::Signal,
    unistd::{close, execve, Pid},
};

use crate::{
    capabilities::setcapabilities, config::ContainerOpts, hostname::set_container_hostname,
    mounts::setmountpoint, namespaces::userns, syscalls::setsyscalls,
};

const STACK_SIZE: usize = 1024 * 1024;

fn child(config: ContainerOpts) -> isize {
    setup_container_configurations(&config).unwrap();
    log::info!(
        "Starting container with command {} and args {:?}",
        config.path.to_str().unwrap(),
        config.argv
    );

    match execve::<CString, CString>(&config.path, &config.argv, &[]) {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

pub(crate) fn generate_child_process(config: ContainerOpts) -> nix::Result<Pid> {
    let mut tmp_stack: [u8; STACK_SIZE] = [0; STACK_SIZE];
    let mut flags = CloneFlags::empty();
    flags.insert(CloneFlags::CLONE_NEWNS);
    flags.insert(CloneFlags::CLONE_NEWCGROUP);
    flags.insert(CloneFlags::CLONE_NEWPID);
    flags.insert(CloneFlags::CLONE_NEWIPC);
    flags.insert(CloneFlags::CLONE_NEWNET);
    flags.insert(CloneFlags::CLONE_NEWUTS);
    clone(
        Box::new(|| child(config.clone())),
        &mut tmp_stack,
        flags,
        Some(Signal::SIGCHLD as i32),
    )
}

fn setup_container_configurations(config: &ContainerOpts) -> Result<(), String> {
    set_container_hostname(&config.hostname)
        .map_err(|err| format!("Could not set container hostname: {:?}", err))?;
    setmountpoint(&config.mount_dir, &config.addpaths)
        .map_err(|err| format!("Could not set mount point: {:?}", err))?;
    userns(config.fd, config.uid);
    close(config.fd).unwrap();
    setcapabilities();
    setsyscalls();
    Ok(())
}