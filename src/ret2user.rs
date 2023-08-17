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
