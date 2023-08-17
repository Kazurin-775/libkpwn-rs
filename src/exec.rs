use std::{ffi::CStr, process::Command};

use nix::unistd::ForkResult;

pub const BIN_SH: &CStr = cstr::cstr!(b"/bin/sh");

pub fn execve_sh() -> nix::Result<std::convert::Infallible> {
    nix::unistd::execve(BIN_SH, &[&BIN_SH], &[] as &[&CStr])
}

pub fn spawn_sh() -> std::io::Result<()> {
    let mut child = Command::new("/bin/sh").spawn()?;
    match child.wait() {
        Ok(status) => {
            log::debug!("Child process exited with {}", status);
        }
        Err(err) => {
            log::error!("Failed to wait for child process: {}", err);
        }
    }
    Ok(())
}

pub fn fork_and_wait(worker: impl FnOnce() -> std::convert::Infallible) -> nix::Result<()> {
    let result = unsafe { nix::unistd::fork() }?;
    match result {
        ForkResult::Parent { child: pid } => {
            log::debug!("Spawned child PID {}", pid);
            let status = nix::sys::wait::waitpid(pid, None);
            match status {
                Ok(status) => {
                    log::debug!("Child process exited with {:?}", status);
                }
                Err(err) => {
                    log::error!("Failed to wait for child process: {}", err);
                }
            }
            Ok(())
        }
        ForkResult::Child => {
            worker();
            unreachable!()
        }
    }
}

pub fn exit_now(code: i32) {
    unsafe {
        nix::libc::_exit(code);
    }
}
