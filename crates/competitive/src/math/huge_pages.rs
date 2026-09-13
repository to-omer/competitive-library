#[inline]
pub fn advise_huge_pages<T>(_values: &mut Vec<T>) {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    unsafe {
        unsafe extern "C" {
            fn madvise(
                addr: *mut std::ffi::c_void,
                len: usize,
                advice: std::ffi::c_int,
            ) -> std::ffi::c_int;
        }
        const PAGE_SIZE: usize = 1 << 12;
        const MADV_HUGEPAGE: std::ffi::c_int = 14;
        let ptr = _values.as_mut_ptr().cast::<u8>();
        let offset = ptr.align_offset(PAGE_SIZE);
        let len = (_values.capacity() * size_of::<T>()).saturating_sub(offset) & !(PAGE_SIZE - 1);
        if len >= 1 << 20 {
            // The hint covers only owned pages; initialization also works without it.
            let _ = madvise(ptr.add(offset).cast(), len, MADV_HUGEPAGE);
        }
    }
}
