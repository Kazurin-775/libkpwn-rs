use std::{cell::UnsafeCell, convert::Infallible};

unsafe extern "C" fn restore_registers(buf: *mut usize) -> ! {
    std::arch::asm!(
        "mov   rsp, [rdi + 0x08]",
        "mov   rbp, [rdi + 0x10]",
        "mov   rbx, [rdi + 0x18]",
        "mov   r12, [rdi + 0x20]",
        "mov   r13, [rdi + 0x28]",
        "mov   r14, [rdi + 0x30]",
        "mov   r15, [rdi + 0x38]",

        "mov   eax, 1",
        "jmp   qword ptr [rdi]",

        in("rdi") buf,
        options(noreturn),
    );
}

#[repr(C)]
pub struct Setjmp {
    regs: UnsafeCell<[usize; 8]>,
}

unsafe impl Send for Setjmp {}
unsafe impl Sync for Setjmp {}

impl Setjmp {
    pub const fn new() -> Setjmp {
        Setjmp {
            regs: UnsafeCell::new([0; 8]),
        }
    }

    pub fn set_and_run(&self, f: impl FnOnce() -> Infallible) {
        let status: i32;

        // Save all callee-saved registers to self.regs.
        // This assembly block must be inlined into this function in order
        // to make it work properly.
        // If it is otherwise moved to a standalone extern "C" function, then
        // the function's return address may get corrupted by the execution of
        // f(), which could result in very strange segfaults (or even worse)
        // upon `self.resume_from_ckpt()`.
        unsafe {
            std::arch::asm!(
                // So THIS WORKS!! Thanks to the great x86_64 architecture...
                "lea   rax, [rip + 3f]",
                "mov   [rdi], rax",
                "mov   [rdi + 0x08], rsp",
                "mov   [rdi + 0x10], rbp",
                "mov   [rdi + 0x18], rbx",
                "mov   [rdi + 0x20], r12",
                "mov   [rdi + 0x28], r13",
                "mov   [rdi + 0x30], r14",
                "mov   [rdi + 0x38], r15",

                "mov   eax, 0",
                "3:",

                in("rdi") self.regs.get(),
                lateout("eax") status,
                clobber_abi("C"),
            );
        }

        if status == 0 {
            f();
            unreachable!();
        } else {
            // log::debug!("Returned from saved state");
            // Don't let the destructor of f cause any problems
            std::mem::forget(f);
        }
    }

    pub unsafe fn resume_from_ckpt(&self) -> ! {
        restore_registers(self.regs.get().cast());
    }
}

#[test]
fn test_setjmp() {
    let sj = Setjmp::new();
    let mut result = 0;
    sj.set_and_run(|| {
        result = 42;
        std::hint::black_box(&result);
        unsafe {
            sj.resume_from_ckpt();
        }
    });
    // Any setjmp operations in Rust will result in very strange compiler
    // optimizations, which may cause tests to fail when built in release mode.
    // Here, we use `black_box()` on references of `result` to force the Rust
    // compiler to treat it as a volatile variable.
    std::hint::black_box(&mut result);
    assert_eq!(result, 42);
}
