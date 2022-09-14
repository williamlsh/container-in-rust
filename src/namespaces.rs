use anyhow::{bail, Result};
use log::{debug, info};
use nix::{
    sched::{unshare, CloneFlags},
    unistd::{setgroups, setresgid, setresuid, Gid, Pid, Uid},
};
use std::{fs::File, io::Write, os::unix::io::RawFd};

use crate::ipc::{recv_boolean, send_boolean};

const USERNS_OFFSET: u64 = 10000;
const USERNS_COUNT: u64 = 2000;

pub fn userns(fd: RawFd, uid: u32) -> Result<()> {
    debug!("Setting up user namespace with UID {}", uid);
    let has_userns = unshare(CloneFlags::CLONE_NEWUSER).is_ok();
    send_boolean(fd, has_userns)?;
    if recv_boolean(fd)? {
        bail!("User namespace isolation isn’t supported");
    }
    if has_userns {
        info!("User namespaces set up");
    } else {
        info!("User namespaces not supported, continuing...");
    }

    debug!("Switching to uid {} / gid {}...", uid, uid);
    let gid = Gid::from_raw(uid);
    let uid = Uid::from_raw(uid);
    setgroups(&[gid])?;
    setresgid(gid, gid, gid)?;
    setresuid(uid, uid, uid)?;
    Ok(())
}

pub fn handle_child_uid_map(pid: Pid, fd: RawFd) -> Result<()> {
    if recv_boolean(fd)? {
        // Perform UID / GID map here
        let mut uid_map = File::create(format!("/proc/{}/{}", pid.as_raw(), "uid_map"))?;
        uid_map.write_all(format!("0 {} {}", USERNS_OFFSET, USERNS_COUNT).as_bytes())?;
        let mut uid_map = File::create(format!("/proc/{}/{}", pid.as_raw(), "gid_map"))?;
        uid_map.write_all(format!("0 {} {}", USERNS_OFFSET, USERNS_COUNT).as_bytes())?;
    } else {
        info!("No user namespace set up from child process");
    }
    log::debug!("Child UID/GID map done, sending signal to child to continue...");
    send_boolean(fd, false)?;
    Ok(())
}
