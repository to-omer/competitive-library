use std::alloc::{GlobalAlloc, Layout, System};

/// A system allocator that aligns allocations of at least 2 MiB for transparent huge pages
/// on Linux x86-64. Select it with `#[global_allocator]` or `main!(huge_pages; ...)`.
/// Large allocations are rounded up to a multiple of 2 MiB; other targets use `System`.
/// The `main!` option requires the `HugePageAllocator` snippet alongside `main`.
pub struct HugePageAllocator;

unsafe impl GlobalAlloc for HugePageAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let Some(layout) = page_layout(layout) else {
            return std::ptr::null_mut();
        };
        unsafe {
            let ptr = System.alloc(layout);
            #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
            if !ptr.is_null() && layout.size() >= 1 << 21 {
                unsafe extern "C" {
                    fn madvise(addr: *mut std::ffi::c_void, len: usize, advice: i32) -> i32;
                }
                // Failure of this placement hint does not affect the allocated memory.
                let _ = madvise(ptr.cast(), layout.size(), 14);
            }
            ptr
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // A successful allocation used this same layout transformation.
        unsafe { System.dealloc(ptr, page_layout(layout).unwrap_unchecked()) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        unsafe {
            if !use_huge_pages(layout.size()) {
                return System.alloc_zeroed(layout);
            }
            let ptr = self.alloc(layout);
            if !ptr.is_null() {
                std::ptr::write_bytes(ptr, 0, layout.size());
            }
            ptr
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        unsafe {
            if !use_huge_pages(layout.size().max(new_size)) {
                return System.realloc(ptr, layout, new_size);
            }
            let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
            let new_ptr = self.alloc(new_layout);
            if !new_ptr.is_null() {
                std::ptr::copy_nonoverlapping(ptr, new_ptr, layout.size().min(new_size));
                self.dealloc(ptr, layout);
            }
            new_ptr
        }
    }
}

fn page_layout(layout: Layout) -> Option<Layout> {
    if use_huge_pages(layout.size()) {
        return layout
            .align_to(1 << 21)
            .ok()
            .map(|layout| layout.pad_to_align());
    }
    Some(layout)
}

fn use_huge_pages(size: usize) -> bool {
    cfg!(all(target_os = "linux", target_arch = "x86_64")) && size >= 1 << 21
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocation() {
        let sizes = [1, 4096, (1 << 21) - 1, 1 << 21, (1 << 21) + 1, 1 << 22];
        for align in [1, 64, 1 << 22] {
            for &size in &sizes {
                unsafe {
                    let layout = Layout::from_size_align(size, align).unwrap();
                    let mut ptr = HugePageAllocator.alloc_zeroed(layout);
                    assert!(!ptr.is_null());
                    assert_eq!(ptr.addr() % align, 0);
                    assert!(
                        std::slice::from_raw_parts(ptr, size)
                            .iter()
                            .all(|&x| x == 0)
                    );
                    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
                    if size >= 1 << 21 {
                        assert_eq!(ptr.addr() % (1 << 21), 0);
                    }
                    std::ptr::write_bytes(ptr, 0x5a, size);
                    let mut old_size = size;
                    for &new_size in &sizes {
                        let layout = Layout::from_size_align(old_size, align).unwrap();
                        let new_ptr = HugePageAllocator.realloc(ptr, layout, new_size);
                        assert!(!new_ptr.is_null());
                        assert_eq!(new_ptr.addr() % align, 0);
                        assert!(
                            std::slice::from_raw_parts(new_ptr, old_size.min(new_size))
                                .iter()
                                .all(|&x| x == 0x5a)
                        );
                        std::ptr::write_bytes(new_ptr, 0x5a, new_size);
                        ptr = new_ptr;
                        old_size = new_size;
                    }
                    HugePageAllocator
                        .dealloc(ptr, Layout::from_size_align(old_size, align).unwrap());
                }
            }
        }
    }
}
