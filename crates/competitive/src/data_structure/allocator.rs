use std::{
    fmt::{self, Debug},
    marker::PhantomData,
    mem::{replace, size_of, take},
    ptr::{self, NonNull, read, write},
};

pub trait Allocator<T> {
    fn allocate(&mut self, value: T) -> NonNull<T>;
    fn deallocate(&mut self, ptr: NonNull<T>) -> T;
}

pub struct MemoryPool<T> {
    pool: Vec<T>,
    chunks: Vec<Vec<T>>,
    unused: Vec<NonNull<T>>,
}

impl<T> Debug for MemoryPool<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Deallocated slots no longer contain values that can be formatted.
        f.debug_struct("MemoryPool")
            .field("pool_len", &self.pool.len())
            .field("chunks", &self.chunks.len())
            .field("unused", &self.unused.len())
            .finish()
    }
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
        if size_of::<T>() == 0 {
            let mut unused = self.unused.len();
            for chunk in std::iter::once(&mut self.pool).chain(&mut self.chunks) {
                let removed = unused.min(chunk.len());
                unsafe { chunk.set_len(chunk.len() - removed) };
                unused -= removed;
            }
            return;
        }
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
            // Higher holes must be removed before they can become a copy source.
            for &p in removed.iter().rev() {
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
        if let Some(ptr) = self.unused.pop() {
            unsafe { write(ptr.as_ptr(), value) };
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
    use std::cell::Cell;

    #[test]
    fn test_alloc() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            let mut pool = MemoryPool::with_capacity(rng.random(0..=32));
            let mut live: Vec<(NonNull<u64>, u64)> = Vec::new();
            for _ in 0..1000 {
                if !live.is_empty() && rng.random(0..3) == 0 {
                    let i = rng.random(0..live.len());
                    let (ptr, value) = live.swap_remove(i);
                    assert_eq!(pool.deallocate(ptr), value);
                } else {
                    let value = rng.rand64();
                    live.push((pool.allocate(value), value));
                }
                for &(ptr, value) in &live {
                    assert_eq!(unsafe { *ptr.as_ref() }, value);
                }
            }
        }
    }

    #[test]
    fn test_drop() {
        struct CheckDrop<'a> {
            id: usize,
            counts: &'a [Cell<usize>],
        }
        impl Drop for CheckDrop<'_> {
            fn drop(&mut self) {
                self.counts[self.id].set(self.counts[self.id].get() + 1);
            }
        }
        // All hole patterns include adjacent holes, the last slot, and multiple chunks.
        for capacity in 0..=5 {
            for n in 0..=8 {
                for mask in 0..1 << n {
                    let counts = vec![Cell::new(0); n];
                    let mut pool = MemoryPool::with_capacity(capacity);
                    let pointers: Vec<_> = (0..n)
                        .map(|id| {
                            pool.allocate(CheckDrop {
                                id,
                                counts: &counts,
                            })
                        })
                        .collect();
                    for (id, &ptr) in pointers.iter().enumerate() {
                        if mask >> id & 1 != 0 {
                            drop(pool.deallocate(ptr));
                        }
                        assert_eq!(counts[id].get(), mask >> id & 1);
                    }
                    drop(pool);
                    assert!(counts.iter().all(|count| count.get() == 1));
                }
            }
        }
    }

    #[test]
    fn test_drop_owned() {
        let mut pool = MemoryPool::with_capacity(3);
        let mut live: Vec<_> = (0..12).map(|i| pool.allocate(Box::new(i))).collect();
        for i in [11, 9, 8, 5, 2, 0] {
            let ptr = live.remove(i);
            drop(pool.deallocate(ptr));
        }
        drop(format!("{pool:?}"));
        for i in 12..24 {
            live.push(pool.allocate(Box::new(i)));
        }
        let expected = [1, 3, 4, 6, 7, 10];
        for (&ptr, value) in live.iter().zip(expected.into_iter().chain(12..24)) {
            assert_eq!(unsafe { **ptr.as_ref() }, value);
        }
        for i in [17, 15, 12, 9, 5, 2, 0] {
            drop(pool.deallocate(live.remove(i)));
        }
        drop(format!("{pool:?}"));
    }

    #[test]
    fn test_drop_zst() {
        #[repr(align(64))]
        struct CheckDrop;
        thread_local! {
            static COUNT: Cell<usize> = const { Cell::new(0) };
        }
        impl Drop for CheckDrop {
            fn drop(&mut self) {
                COUNT.with(|count| count.set(count.get() + 1));
            }
        }
        COUNT.with(|count| count.set(0));
        let mut pool = MemoryPool::with_capacity(0);
        let pointers: Vec<_> = (0..4).map(|_| pool.allocate(CheckDrop)).collect();
        for &ptr in &pointers[..3] {
            drop(pool.deallocate(ptr));
        }
        pool.allocate(CheckDrop);
        drop(pool);
        assert_eq!(COUNT.with(Cell::get), 5);
    }
}
