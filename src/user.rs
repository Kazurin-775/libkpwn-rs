/// Print the current process's UID, GID and supplementary groups info, as what
/// the `id` command will do.
pub fn whoami() {
    use nix::unistd::*;
    log::info!("Who am I:");
    // getres{u,g}id() should never fail (as long as the caller passes
    // valid pointers to these functions)
    log::info!("UID = {:?}", getresuid().unwrap());
    log::info!("GID = {:?}", getresgid().unwrap());
    log::info!("Groups = {:?}", getgroups());
}

/// Become root after overwriting `struct cred` of the current process.
pub fn su_root() -> nix::Result<()> {
    use nix::unistd::*;

    let uid_root = Uid::from_raw(0);
    setresuid(uid_root, uid_root, uid_root)?;

    // `setresgid()` failures are not considered critical
    let gid_root = Gid::from_raw(0);
    if let Err(err) = setresgid(gid_root, gid_root, gid_root) {
        log::warn!("setresgid(0, 0, 0) failed: {}", err);
    }

    // `setgroups(&[])` may fail if we only manage to overwrite those UID and
    // GID fields in `struct cred` (which itself does not update the process's
    // capabilities set; a call to either `capset()` or `execve()` is required
    // to perform the update).
    // TODO: call `capset()` here to update the process's capabilities
    if let Err(err) = setgroups(&[]) {
        log::warn!("setgroups([]) failed: {}", err);
    }

    Ok(())
}
