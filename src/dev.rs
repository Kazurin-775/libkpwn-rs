use std::{fs::File, os::fd::FromRawFd, path::Path};

use nix::{fcntl::OFlag, sys::stat::Mode};

pub fn open_dev<P>(path: &P) -> nix::Result<File>
where
    P: AsRef<Path> + ?Sized,
{
    let fd = nix::fcntl::open(
        path.as_ref(),
        // O_CLOEXEC is omitted (which is different from Rust's std::fs module)
        OFlag::O_RDWR,
        Mode::from_bits_truncate(0o666),
    )
    .unwrap();
    Ok(unsafe { File::from_raw_fd(fd) })
}

pub fn open_ptmx() -> nix::Result<File> {
    let fd = nix::fcntl::open(
        "/dev/ptmx",
        OFlag::O_RDWR | OFlag::O_NOCTTY,
        Mode::from_bits_truncate(0o666),
    )
    .unwrap();
    Ok(unsafe { File::from_raw_fd(fd) })
}
