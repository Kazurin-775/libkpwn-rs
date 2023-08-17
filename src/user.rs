pub fn whoami() {
    use nix::unistd::*;
    log::info!("Who am I:");
    // getres{u,g}id() should never fail (as long as the caller passes
    // valid pointers to these functions)
    log::info!("UID = {:?}", getresuid().unwrap());
    log::info!("GID = {:?}", getresgid().unwrap());
    log::info!("Groups = {:?}", getgroups());
}
