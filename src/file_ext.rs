use std::{
    mem::MaybeUninit,
    os::fd::{AsRawFd, IntoRawFd},
};

use nix::libc::SYS_ioctl;

pub trait FileExt {
    unsafe fn read_raw<D>(&self, buf: *mut D, len: usize) -> std::io::Result<usize>;
    unsafe fn write_raw<D>(&self, buf: *const D, len: usize) -> std::io::Result<usize>;
    unsafe fn read_all_raw<D>(&self, buf: *mut D) -> std::io::Result<()>;
    unsafe fn write_all_raw<D>(&self, buf: *const D) -> std::io::Result<()>;
    unsafe fn read_struct<S>(&self) -> std::io::Result<S>;

    unsafe fn ioctl_raw(&self, cmd: u64, arg: u64) -> i64;

    fn close(self) -> std::io::Result<()>;
    fn dont_close(self);
}

impl FileExt for std::fs::File {
    unsafe fn read_raw<D>(&self, buf: *mut D, len: usize) -> std::io::Result<usize> {
        nix::unistd::read(
            self.as_raw_fd(),
            std::slice::from_raw_parts_mut(buf as *mut u8, len),
        )
        .map_err(Into::into)
    }

    unsafe fn write_raw<D>(&self, buf: *const D, len: usize) -> std::io::Result<usize> {
        nix::unistd::write(
            self.as_raw_fd(),
            std::slice::from_raw_parts(buf as *const u8, len),
        )
        .map_err(Into::into)
    }

    unsafe fn read_all_raw<D>(&self, buf: *mut D) -> std::io::Result<()> {
        let bytes_read = self.read_raw(buf, std::mem::size_of::<D>())?;
        assert_eq!(bytes_read, std::mem::size_of::<D>());
        Ok(())
    }

    unsafe fn write_all_raw<D>(&self, buf: *const D) -> std::io::Result<()> {
        let bytes_written = self.write_raw(buf, std::mem::size_of::<D>())?;
        assert_eq!(bytes_written, std::mem::size_of::<D>());
        Ok(())
    }

    unsafe fn read_struct<S>(&self) -> std::io::Result<S> {
        let mut data = MaybeUninit::uninit();
        self.read_all_raw(data.as_mut_ptr())?;
        Ok(data.assume_init())
    }

    fn close(self) -> std::io::Result<()> {
        let fd = self.into_raw_fd();
        nix::unistd::close(fd).map_err(Into::into)
    }

    fn dont_close(self) {
        self.into_raw_fd();
    }

    unsafe fn ioctl_raw(&self, cmd: u64, arg: u64) -> i64 {
        let fd = self.as_raw_fd();
        let result: i64;

        std::arch::asm!(
            "syscall",

            in("rax") SYS_ioctl,
            in("rdi") fd,
            in("rsi") cmd,
            in("rdx") arg,
            lateout("rax") result,
            out("rcx") _,
            out("r11") _,

            options(nostack),
        );

        result
    }
}
