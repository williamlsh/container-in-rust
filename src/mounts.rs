use anyhow::Result;
use nix::{
    mount::{mount, umount2, MntFlags, MsFlags},
    unistd::{chdir, pivot_root},
};
use rand::Rng;
use std::{
    fs::{create_dir_all, remove_dir},
    path::{Path, PathBuf},
};

pub(crate) fn setmountpoint(mount_dir: &PathBuf, addpaths: &[(PathBuf, PathBuf)]) -> Result<()> {
    log::debug!("Setting mount points...");
    mount_directory(
        None,
        &PathBuf::from("/"),
        vec![MsFlags::MS_REC, MsFlags::MS_PRIVATE],
    )?;

    let new_root = PathBuf::from(format!("/tmp/crabcan.{}", random_string(12)));
    log::debug!("Mounting temp directory {}", new_root.to_str().unwrap());
    create_dir_all(&new_root)?;
    mount_directory(
        Some(mount_dir),
        &new_root,
        vec![MsFlags::MS_BIND, MsFlags::MS_PRIVATE],
    )?;

    log::debug!("Mounting additionnal paths");
    for (inpath, mntpath) in addpaths.iter() {
        let outpath = new_root.join(mntpath);
        create_dir_all(&outpath)?;
        mount_directory(
            Some(inpath),
            &outpath,
            vec![MsFlags::MS_PRIVATE, MsFlags::MS_BIND],
        )?;
    }

    log::debug!("Pivoting root");
    let old_root_tail = format!("oldroot.{}", random_string(6));
    let put_old = new_root.join(PathBuf::from(old_root_tail.clone()));
    create_dir_all(&put_old)?;
    pivot_root(&new_root, &put_old)?;

    log::debug!("Unmounting old root");
    let old_root = PathBuf::from(format!("/{}", old_root_tail));
    chdir(&PathBuf::from("/"))?;
    umount_path(&old_root)?;
    remove_dir(&old_root)?;

    Ok(())
}

pub(crate) fn clean_mounts(rootPath: &Path) -> nix::Result<()> {
    Ok(())
}

fn mount_directory(
    path: Option<&PathBuf>,
    mount_point: &PathBuf,
    flags: Vec<MsFlags>,
) -> nix::Result<()> {
    let mut ms_flags = MsFlags::empty();
    for f in flags {
        ms_flags.insert(f);
    }
    mount::<PathBuf, PathBuf, PathBuf, PathBuf>(path, mount_point, None, ms_flags, None)
}

fn random_string(n: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                             abcdefghijklmnopqrstuvwxyz\
                             0123456789";
    let mut rng = rand::thread_rng();
    (0..n)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

fn umount_path(path: &PathBuf) -> nix::Result<()> {
    umount2(path, MntFlags::MNT_DETACH)
}
