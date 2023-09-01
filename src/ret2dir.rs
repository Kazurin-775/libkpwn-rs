use std::{
    num::NonZeroUsize,
    ops::{Index, IndexMut},
};

use nix::sys::mman::{MapFlags, ProtFlags};

pub struct MmapHandle {
    ptr: *mut MmapPage,
    num_pages: usize,
}

#[repr(C)]
pub struct MmapPage {
    content: [u8; 1 << 12],
}

impl MmapHandle {
    pub fn alloc_pages(num_pages: usize) -> nix::Result<MmapHandle> {
        assert_ne!(num_pages, 0);
        // Safety: as long as the `mmap()` operation succeeds, the resulting
        // memory region is guaranteed to be 4k-aligned and zero-initialized.
        let ptr = unsafe {
            nix::sys::mman::mmap(
                None,
                NonZeroUsize::new(num_pages << 12).unwrap(),
                ProtFlags::all(),
                MapFlags::MAP_ANONYMOUS | MapFlags::MAP_PRIVATE,
                -1,
                0,
            )
        }?;
        Ok(MmapHandle {
            ptr: ptr.cast(),
            num_pages,
        })
    }

    pub fn get_page(&mut self, index: usize) -> &mut MmapPage {
        &mut self[index]
    }

    pub fn first_page(&mut self) -> &mut MmapPage {
        &mut self[0]
    }

    pub fn copy_first_page_to_others(&mut self) {
        // Safety: see `impl Index<usize> for MmapHandle`.
        let src_page = unsafe { &(*self.ptr).content };
        for dest in 1..self.num_pages {
            let dest_page = unsafe { &mut (*self.ptr.add(dest)).content };
            dest_page.copy_from_slice(src_page);
        }
    }
}

impl Index<usize> for MmapHandle {
    type Output = MmapPage;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.num_pages);
        // Safety: bounds checking is correctly applied, and this does not
        // violate Rust's memory aliasing rules in any way.
        unsafe { &*self.ptr.add(index) }
    }
}

impl IndexMut<usize> for MmapHandle {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.num_pages);
        // Safety: see `impl Index<usize> for MmapHandle`.
        unsafe { &mut *self.ptr.add(index) }
    }
}

impl Drop for MmapHandle {
    fn drop(&mut self) {
        // Safety: we own the pointer, and since `drop()` is being called,
        // it is guaranteed that nobody except us is using the memory that
        // it points to.
        unsafe {
            if let Err(err) = nix::sys::mman::munmap(self.ptr.cast(), self.num_pages << 12) {
                log::error!("Failed to release MmapHandle: {}", err);
            }
        }
    }
}

impl MmapPage {
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.content
    }

    pub fn as_u64_array_mut(&mut self) -> &mut [u64] {
        // Safety: yeah, we do need to reinterpret the underlying memory in
        // this scenario, but since we are doing this for kernel pwning,
        // how could that be a problem for us anyway?
        unsafe { std::slice::from_raw_parts_mut(self.content.as_mut_ptr().cast(), 1 << 12 >> 3) }
    }

    pub fn init_with_u64_array(&mut self, array: &[u64]) {
        let dest = self.as_u64_array_mut();
        assert!(array.len() <= dest.len());
        dest[0..array.len()].copy_from_slice(array);
    }

    pub fn init_with_struct<S>(&mut self, data: &S) {
        // Safety: almost the same as `as_u64_array_mut()`. However, in this
        // case, the caller should also make sure that the struct `S` is
        // `memcpy()`able (i.e. contains only pure data).
        // (But it's perfectly okay if it is not, since we won't dereference
        // the destination struct anyway.)
        // This shouldn't be a problem in the context of kernel pwning.
        let data = unsafe {
            std::slice::from_raw_parts(data as *const S as *const u8, std::mem::size_of::<S>())
        };
        let dest = self.as_bytes_mut();
        assert!(data.len() <= dest.len());
        dest[0..data.len()].copy_from_slice(data);
    }
}
