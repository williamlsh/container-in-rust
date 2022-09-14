use anyhow::Result;
use std::{ffi::CString, os::unix::prelude::RawFd, path::PathBuf};

use crate::{hostname::generate_hostname, ipc::generate_socketpair};

#[derive(Debug, Clone)]
pub(crate) struct ContainerOpts {
    // The path of the binary/executable/script to execute inside the container.
    pub(crate) path: CString,

    // The full arguments passed (including the 'path' option) into the command line.
    pub(crate) argv: Vec<CString>,

    // The ID of the user inside the container. An ID of '0' means it’s root (administrator).
    pub(crate) uid: u32,

    // The path of the directory we want to use as a '/' root inside our container.
    pub(crate) mount_dir: PathBuf,

    pub(crate) fd: RawFd,

    pub(crate) hostname: String,

    /// Mount a directory inside the container
    pub addpaths: Vec<(PathBuf, PathBuf)>,
}

impl ContainerOpts {
    pub(crate) fn new(
        command: String,
        uid: u32,
        mount_dir: PathBuf,
        addpaths: Vec<(PathBuf, PathBuf)>,
    ) -> Result<(ContainerOpts, (RawFd, RawFd))> {
        let argv: Vec<CString> = command
            .split_ascii_whitespace()
            .map(|s| CString::new(s).unwrap_or_else(|_| panic!("Could not read arg: {}", s)))
            .collect();
        let path = argv[0].clone();

        let sockets = generate_socketpair().expect("Failed to generate socket pair");

        Ok((
            ContainerOpts {
                path,
                argv,
                uid,
                mount_dir,
                fd: sockets.1,
                hostname: generate_hostname()?,
                addpaths,
            },
            sockets,
        ))
    }
}
