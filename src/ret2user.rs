#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct TrapFrame64 {
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

impl TrapFrame64 {
    pub fn new(rip: extern "C" fn() -> !) -> TrapFrame64 {
        let rsp: u64;
        unsafe {
            std::arch::asm!(
                "mov {}, rsp",
                out(reg) rsp,
                options(nomem, nostack),
            );
        }

        TrapFrame64 {
            rip: rip as usize as u64,
            cs: 0x33,
            rflags: 0x202, // only IF (and an always-1 bit) is set
            // The C calling convention requires the least hex byte of sp to be
            // 0x8 upon function entry (i.e. after the "call" instruction)
            rsp: (rsp - 0x1_000) & !0xF | 8,
            ss: 0x2B,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct PtRegs {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbp: u64,
    pub rbx: u64,
    pub r11_clob: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rax_clob: u64,
    pub rcx_clob: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
}

impl PtRegs {
    pub fn bogus() -> PtRegs {
        PtRegs {
            r15: 0xDEAD_BEEF_CAFE_0001,
            r14: 0xDEAD_BEEF_CAFE_0002,
            r13: 0xDEAD_BEEF_CAFE_0003,
            r12: 0xDEAD_BEEF_CAFE_0004,
            rbp: 0xDEAD_BEEF_CAFE_0005,
            rbx: 0xDEAD_BEEF_CAFE_0006,
            r11_clob: 0xDEAD_BEEF_CAFE_0007,
            r10: 0xDEAD_BEEF_CAFE_0008,
            r9: 0xDEAD_BEEF_CAFE_0009,
            r8: 0xDEAD_BEEF_CAFE_000A,
            rax_clob: 0xDEAD_BEEF_CAFE_000B,
            rcx_clob: 0xDEAD_BEEF_CAFE_000C,
            rdx: 0xDEAD_BEEF_CAFE_000D,
            rsi: 0xDEAD_BEEF_CAFE_000E,
            rdi: 0xDEAD_BEEF_CAFE_000F,
        }
    }
}

pub unsafe fn ioctl_with_regs(regs: &PtRegs) -> i64 {
    let result;

    std::arch::asm!(
        // Callee-preserved registers
        "push rbx",
        "push rbp",
        "push r12",
        "push r13",
        "push r14",
        "push r15",

        // Load registers from pt_regs
        "mov  r15, [rdi + 0x00]",
        "mov  r14, [rdi + 0x08]",
        "mov  r13, [rdi + 0x10]",
        "mov  r12, [rdi + 0x18]",
        "mov  rbp, [rdi + 0x20]",
        "mov  rbx, [rdi + 0x28]",
        // "mov  r11, [rdi + 0x30]",
        "mov  r10, [rdi + 0x38]",
        "mov  r9,  [rdi + 0x40]",
        "mov  r8,  [rdi + 0x48]",
        // "mov  rax, [rdi + 0x50]",
        // "mov  rcx, [rdi + 0x58]",
        "mov  rdx, [rdi + 0x60]",
        "mov  rsi, [rdi + 0x68]",
        "mov  rdi, [rdi + 0x70]",

        "syscall",

        // Restore registers
        "pop  r15",
        "pop  r14",
        "pop  r13",
        "pop  r12",
        "pop  rbp",
        "pop  rbx",

        in("rax") nix::libc::SYS_ioctl,
        in("rdi") regs,
        lateout("rax") result,
        lateout("rdi") _,

        // Clobber list
        out("r11") _,
        out("r10") _,
        out("r9") _,
        out("r8") _,
        out("rcx") _,
        out("rdx") _,
        out("rsi") _,
    );

    result
}
