use std::{fs::File, io::Write, os::unix::io::RawFd};

use log::{debug, error, info};
use nix::{
    sched::{unshare, CloneFlags},
    unistd::{setgroups, setresgid, setresuid, Gid, Pid, Uid},
};

use crate::ipc::{recv_boolean, send_boolean};

const USERNS_OFFSET: u64 = 10000;
const USERNS_COUNT: u64 = 2000;

pub fn userns(fd: RawFd, uid: u32) {
    debug!("Setting up user namespace with UID {}", uid);
    let has_userns = unshare(CloneFlags::CLONE_NEWUSER).is_ok();
    send_boolean(fd, has_userns).unwrap();
    if recv_boolean(fd).unwrap() {
        error!("User namespace isolation isn’t supported");
        return;
    }
    if has_userns {
        info!("User namespaces set up");
    } else {
        info!("User namespaces not supported, continuing...");
    }

    debug!("Switching to uid {} / gid {}...", uid, uid);
    let gid = Gid::from_raw(uid);
    let uid = Uid::from_raw(uid);
    setgroups(&[gid]).unwrap();
    setresgid(gid, gid, gid).unwrap();
    setresuid(uid, uid, uid).unwrap();
}

pub fn handle_child_uid_map(pid: Pid, fd: RawFd) {
    if recv_boolean(fd).unwrap() {
        // Perform UID / GID map here
        File::create(format!("/proc/{}/{}", pid.as_raw(), "uid_map"))
            .map(|mut uid_map| {
                uid_map
                    .write_all(format!("0 {} {}", USERNS_OFFSET, USERNS_COUNT).as_bytes())
                    .unwrap()
            })
            .unwrap();
        File::create(format!("/proc/{}/{}", pid.as_raw(), "gid_map"))
            .map(|mut uid_map| {
                uid_map
                    .write_all(format!("0 {} {}", USERNS_OFFSET, USERNS_COUNT).as_bytes())
                    .unwrap()
            })
            .unwrap();
    } else {
        info!("No user namespace set up from child process");
    }
    log::debug!("Child UID/GID map done, sending signal to child to continue...");
    send_boolean(fd, false).unwrap()
}
