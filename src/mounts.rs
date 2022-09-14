use std::{
    fs::{create_dir_all, remove_dir},
    io,
    path::PathBuf,
};

use nix::{
    mount::{mount, umount2, MntFlags, MsFlags},
    unistd::{chdir, pivot_root},
};
use rand::Rng;

pub(crate) fn setmountpoint(mount_dir: &PathBuf, addpaths: &Vec<(PathBuf, PathBuf)>) -> nix::Result<()> {
    log::debug!("Setting mount point ...");
    mount_directory(
        None,
        &PathBuf::from("/"),
        vec![MsFlags::MS_REC, MsFlags::MS_PRIVATE],
    )?;

    let new_root = PathBuf::from(format!("/tmp/crabcan.{}", random_string(12)));
    log::debug!(
        "Mounting temp directory {}",
        new_root.as_path().to_str().unwrap()
    );
    create_directory(&new_root).unwrap();
    mount_directory(
        Some(mount_dir),
        &new_root,
        vec![MsFlags::MS_BIND, MsFlags::MS_PRIVATE],
    )?;

    log::debug!("Mounting additionnal paths");
    for (inpath, mntpath) in addpaths.iter(){
        let outpath = new_root.join(mntpath);
        create_directory(&outpath).unwrap();
        mount_directory(Some(inpath), &outpath, vec![MsFlags::MS_PRIVATE, MsFlags::MS_BIND])?;
    }

    log::debug!("Pivoting root");
    let old_root_tail = format!("oldroot.{}", random_string(6));
    let put_old = new_root.join(PathBuf::from(old_root_tail.clone()));
    create_directory(&put_old).unwrap();
    pivot_root(&new_root, &put_old)?;

    log::debug!("Unmounting old root");
    let old_root = PathBuf::from(format!("/{}", old_root_tail));
    chdir(&PathBuf::from("/"))?;
    umount_path(&old_root)?;
    delete_dir(&old_root).unwrap();

    Ok(())
}

pub(crate) fn clean_mounts(rootPath: &PathBuf) -> nix::Result<()> {
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
    match mount::<PathBuf, PathBuf, PathBuf, PathBuf>(path, mount_point, None, ms_flags, None) {
        Ok(_) => Ok(()),
        Err(err) => {
            match path {
                Some(p) => log::error!(
                    "Could not mount {} to {}: {}",
                    p.to_str().unwrap(),
                    mount_point.to_str().unwrap(),
                    err
                ),
                None => log::error!(
                    "Could not remount {}: {}",
                    mount_point.to_str().unwrap(),
                    err
                ),
            }
            Err(err)
        }
    }
}

fn random_string(n: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                             abcdefghijklmnopqrstuvwxyz\
                             0123456789";
    let mut rng = rand::thread_rng();
    let name: String = (0..n)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    name
}

fn create_directory(path: &PathBuf) -> Result<(), std::io::Error> {
    match create_dir_all(path) {
        Err(err) => {
            log::error!(
                "Could not create directory {}: {}",
                path.to_str().unwrap(),
                err
            );
            Err(err)
        }
        Ok(_) => Ok(()),
    }
}

fn umount_path(path: &PathBuf) -> nix::Result<()> {
    match umount2(path, MntFlags::MNT_DETACH) {
        Ok(_) => Ok(()),
        Err(err) => {
            log::error!("Unable to umount {}: {}", path.to_str().unwrap(), err);
            Err(err)
        }
    }
}

fn delete_dir(path: &PathBuf) -> io::Result<()> {
    match remove_dir(path.as_path()) {
        Ok(_) => Ok(()),
        Err(err) => {
            log::error!(
                "Unable to delete directory {}: {}",
                path.to_str().unwrap(),
                err
            );
            Err(err)
        }
    }
}
