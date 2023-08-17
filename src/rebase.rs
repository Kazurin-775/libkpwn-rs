#[derive(Debug, Clone, Copy)]
pub struct KernelRebaser {
    pub base: u64,
}

const STD_BASE: u64 = 0xffff_ffff_8000_0000;
const KERNEL_TEXT_MAX_SIZE: u64 = 0x2000_0000;

impl KernelRebaser {
    pub fn new(base: u64) -> KernelRebaser {
        KernelRebaser { base }
    }

    pub fn r(&self, std_addr: u64) -> u64 {
        std_addr - STD_BASE + self.base
    }

    pub fn o(&self, offset: u64) -> u64 {
        self.base + offset
    }

    pub fn fixup(&self, addr: &mut u64) {
        if *addr >= STD_BASE && *addr < STD_BASE + KERNEL_TEXT_MAX_SIZE {
            let before_fixup = *addr;
            *addr = self.r(*addr);
            log::debug!("Fixup kernel addr {:#x} -> {:#x}", before_fixup, *addr);
        }
    }

    pub fn fixup_all<'a, A>(&self, array: A)
    where
        A: IntoIterator<Item = &'a mut u64>,
    {
        for item in array {
            self.fixup(item);
        }
    }

    pub fn rebaser(&self) -> impl Fn(u64) -> u64 {
        let base = self.base;
        move |addr| addr - STD_BASE + base
    }
}

impl std::ops::Add<u64> for KernelRebaser {
    type Output = u64;

    fn add(self, rhs: u64) -> Self::Output {
        self.o(rhs)
    }
}

impl std::ops::Rem<u64> for KernelRebaser {
    type Output = u64;

    fn rem(self, rhs: u64) -> Self::Output {
        self.r(rhs)
    }
}
