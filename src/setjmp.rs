use std::{cell::UnsafeCell, convert::Infallible, mem::ManuallyDrop};

unsafe extern "C" fn save_registers_and_call(buf: *mut usize, fn_addr: usize, arg0: usize) {
    // Save all callee-saved registers to self.regs.
    unsafe {
        std::arch::asm!(
            // So `lea rXX, [rip + some_label]` WORKS!! Thanks to the great
            // x86_64 architecture...
            "lea   rax, [rip + 3f]",
            "mov   [rdi], rax",
            "mov   [rdi + 0x08], rsp",
            "mov   [rdi + 0x10], rbp",
            "mov   [rdi + 0x18], rbx",
            "mov   [rdi + 0x20], r12",
            "mov   [rdi + 0x28], r13",
            "mov   [rdi + 0x30], r14",
            "mov   [rdi + 0x38], r15",

            // Obtain a new stack frame (by re-aligning the stack pointer)
            // and call setjmp_call_rust_fn::<F>(fn_ptr).
            //
            // Note that we don't actually use C's `setjmp() / longjmp()`
            // semantics here, as this will result in undefined behavior
            // in Rust, as discussed here:
            // https://github.com/rust-lang/libc/pull/1216
            //
            // For more information (and how this was previously
            // implemented), see the commit `08cb4023` in this repo.
            "sub   rsp, 8",
            "mov   rdi, rdx",
            "jmp   rsi",

            "3:",

            in("rdi") buf,
            in("rsi") fn_addr,
            in("rdx") arg0,
            clobber_abi("C"),
        );
    }
}

unsafe extern "C" fn setjmp_call_rust_fn<F>(f: *mut ManuallyDrop<F>) -> Infallible
where
    F: FnOnce() -> Infallible,
{
    // "Move" the instance of `F` out of the pointer and into this function's
    // stack frame by abusing the functionality of `ManuallyDrop`.
    let f = ManuallyDrop::take(&mut *f);
    f()
}

unsafe extern "C" fn restore_registers(buf: *mut usize) -> ! {
    std::arch::asm!(
        "mov   rsp, [rdi + 0x08]",
        "mov   rbp, [rdi + 0x10]",
        "mov   rbx, [rdi + 0x18]",
        "mov   r12, [rdi + 0x20]",
        "mov   r13, [rdi + 0x28]",
        "mov   r14, [rdi + 0x30]",
        "mov   r15, [rdi + 0x38]",

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

    pub fn set_and_run<F>(&self, f: F)
    where
        F: FnOnce() -> Infallible,
    {
        // As we are going to move `f` into another function through a pointer,
        // prevent is destructor from being called by this function by using
        // `ManuallyDrop`.
        let mut f = ManuallyDrop::new(f);

        unsafe {
            save_registers_and_call(
                self.regs.get().cast(),
                setjmp_call_rust_fn::<F> as usize,
                &mut f as *mut ManuallyDrop<F> as usize,
            );
        }

        // log::debug!("Returned from saved state");
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

        unsafe {
            sj.resume_from_ckpt();
        }
    });
    assert_eq!(result, 42);
}
