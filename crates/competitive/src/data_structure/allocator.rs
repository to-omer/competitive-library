use std::{
    marker::PhantomData,
    mem::{replace, size_of, take},
    ptr::{self, NonNull, read, write},
};

pub trait Allocator<T> {
    fn allocate(&mut self, value: T) -> NonNull<T>;
    fn deallocate(&mut self, ptr: NonNull<T>) -> T;
}

#[derive(Debug)]
pub struct MemoryPool<T> {
    pool: Vec<T>,
    chunks: Vec<Vec<T>>,
    unused: Vec<NonNull<T>>,
}

impl<T> Default for MemoryPool<T> {
    fn default() -> Self {
        Self::with_capacity(CAP / 1usize.max(size_of::<T>()))
    }
}

const CAP: usize = 1024;

impl<T> MemoryPool<T> {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn with_capacity(capacity: usize) -> Self {
        let pool = Vec::with_capacity(capacity.max(1));
        Self {
            pool,
            chunks: Vec::new(),
            unused: Vec::new(),
        }
    }
}

impl<T> Drop for MemoryPool<T> {
    fn drop(&mut self) {
        self.chunks.push(take(&mut self.pool));
        let mut removed = vec![vec![]; self.chunks.len()];
        for p in self.unused.iter() {
            let p = p.as_ptr();
            for (chunk, removed) in self.chunks.iter().zip(&mut removed).rev() {
                let ptr = chunk.as_ptr() as *mut _;
                let len = chunk.len();
                if ptr <= p && p < ptr.wrapping_add(len) {
                    removed.push(p);
                }
            }
        }
        for (chunk, removed) in self.chunks.iter_mut().zip(&mut removed) {
            removed.sort_unstable();
            for &p in removed.iter() {
                unsafe {
                    let len = chunk.len();
                    let base_ptr = chunk.as_mut_ptr();
                    ptr::copy(base_ptr.add(len - 1), p, 1);
                    chunk.set_len(len - 1);
                }
            }
        }
    }
}

impl<T> Allocator<T> for MemoryPool<T> {
    fn allocate(&mut self, value: T) -> NonNull<T> {
        if let Some(mut ptr) = self.unused.pop() {
            unsafe { write(ptr.as_mut(), value) };
            ptr
        } else {
            let len = self.pool.len();
            if len >= self.pool.capacity() {
                let new_capacity = self.pool.capacity() * 2;
                let new_pool = Vec::with_capacity(new_capacity);
                self.chunks.push(replace(&mut self.pool, new_pool));
            }
            let len = self.pool.len();
            debug_assert!(len < self.pool.capacity());
            self.pool.push(value);
            unsafe { NonNull::new_unchecked(self.pool.as_mut_ptr().add(len)) }
        }
    }

    fn deallocate(&mut self, ptr: NonNull<T>) -> T {
        self.unused.push(ptr);
        unsafe { read(ptr.as_ptr()) }
    }
}

#[derive(Debug)]
pub struct BoxAllocator<T>(PhantomData<fn() -> T>);

impl<T> Default for BoxAllocator<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T> Allocator<T> for BoxAllocator<T> {
    fn allocate(&mut self, value: T) -> NonNull<T> {
        unsafe { NonNull::new_unchecked(Box::leak(Box::new(value))) }
    }
    fn deallocate(&mut self, ptr: NonNull<T>) -> T {
        unsafe { *Box::from_raw(ptr.as_ptr()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;
    use std::cell::RefCell;

    #[test]
    fn test_alloc() {
        let mut pool = MemoryPool::<usize>::with_capacity(2);
        let mut a = vec![];
        for i in 0..100 {
            let p = pool.allocate(i);
            a.push(p);
            for (i, &p) in a.iter().enumerate() {
                assert_eq!(unsafe { *p.as_ref() }, i);
            }
        }
    }

    #[test]
    fn test_drop() {
        #[derive(Debug)]
        struct CheckDrop<T>(T);
        thread_local! {
            static CNT: RefCell<usize> = const { RefCell::new(0) };
        }
        impl<T> Drop for CheckDrop<T> {
            fn drop(&mut self) {
                CNT.with(|cnt| *cnt.borrow_mut() += 1);
            }
        }
        const Q: usize = 100;
        let mut cnt = 0usize;
        let mut rng = Xorshift::default();
        for _ in 0..10 {
            let mut pool = MemoryPool::new();
            let mut a = vec![];
            for _ in 0..Q {
                if a.is_empty() || rng.gen_bool(0.8) {
                    let k = rng.rand(!0);
                    a.push(pool.allocate(CheckDrop(k)));
                } else {
                    let i = rng.rand(a.len() as _) as usize;
                    let p = a.swap_remove(i);
                    pool.deallocate(p);
                    cnt += 1;
                }
                assert_eq!(cnt, CNT.with(|cnt| *cnt.borrow()));
            }
            cnt += a.len();
        }
        assert_eq!(cnt, CNT.with(|cnt| *cnt.borrow()));
    }
}
