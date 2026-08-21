use std::{cmp::Ordering, ptr::copy_nonoverlapping};

pub trait RadixSortKey: Copy {
    const BYTES: usize;
    fn radix_byte(self, byte: usize) -> usize;
}

macro_rules! unsigned_radix_sort_key {
    ($($t:ty),* $(,)?) => {
        $(
            impl RadixSortKey for $t {
                const BYTES: usize = (<$t>::BITS / 8) as usize;

                fn radix_byte(self, byte: usize) -> usize {
                    ((self >> (byte * 8)) & 0xff) as usize
                }
            }
        )*
    };
}

macro_rules! signed_radix_sort_key {
    ($($t:ty => $u:ty),* $(,)?) => {
        $(
            impl RadixSortKey for $t {
                const BYTES: usize = (<$t>::BITS / 8) as usize;

                fn radix_byte(self, byte: usize) -> usize {
                    let key = self as $u ^ (1 << (<$t>::BITS - 1));
                    ((key >> (byte * 8)) & 0xff) as usize
                }
            }
        )*
    };
}

unsigned_radix_sort_key!(u8, u16, u32, u64, u128, usize);
signed_radix_sort_key!(
    i8 => u8,
    i16 => u16,
    i32 => u32,
    i64 => u64,
    i128 => u128,
    isize => usize,
);

pub trait SliceSortExt<T> {
    fn bubble_sort(&mut self)
    where
        T: Ord;
    fn bubble_sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&T, &T) -> Ordering;
    fn merge_sort(&mut self)
    where
        T: Ord;
    fn merge_sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&T, &T) -> Ordering;
    fn insertion_sort(&mut self)
    where
        T: Ord;
    fn insertion_sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&T, &T) -> Ordering;
    fn radix_sort_by_key<K>(&mut self, key: impl FnMut(&T) -> K)
    where
        T: Clone,
        K: RadixSortKey;
}
impl<T> SliceSortExt<T> for [T] {
    fn bubble_sort(&mut self)
    where
        T: Ord,
    {
        bubble_sort(self, |a, b| a.lt(b));
    }
    fn bubble_sort_by<F>(&mut self, mut compare: F)
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        bubble_sort(self, |a, b| compare(a, b) == Ordering::Less);
    }
    fn merge_sort(&mut self)
    where
        T: Ord,
    {
        merge_sort(self, |a, b| a.lt(b));
    }
    fn merge_sort_by<F>(&mut self, mut compare: F)
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        merge_sort(self, |a, b| compare(a, b) == Ordering::Less);
    }
    fn insertion_sort(&mut self)
    where
        T: Ord,
    {
        insertion_sort(self, |a, b| a.lt(b));
    }
    fn insertion_sort_by<F>(&mut self, mut compare: F)
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        insertion_sort(self, |a, b| compare(a, b) == Ordering::Less);
    }
    fn radix_sort_by_key<K>(&mut self, key: impl FnMut(&T) -> K)
    where
        T: Clone,
        K: RadixSortKey,
    {
        radix_sort_by_key(self, key);
    }
}

fn radix_sort_by_key<T, K>(values: &mut [T], mut key: impl FnMut(&T) -> K)
where
    T: Clone,
    K: RadixSortKey,
{
    if values.len() <= 1 {
        return;
    }
    let mut histograms = vec![[0usize; 256]; K::BYTES];
    for value in values.iter() {
        let key = key(value);
        for (byte, histogram) in histograms.iter_mut().enumerate() {
            histogram[key.radix_byte(byte)] += 1;
        }
    }
    for histogram in histograms.iter_mut() {
        let mut position = 0;
        for count in histogram.iter_mut() {
            let next = position + *count;
            *count = position;
            position = next;
        }
    }
    let mut ends = [values.len(); 256];
    ends[..255].copy_from_slice(&histograms[0][1..]);
    let mut buffer = Vec::with_capacity(values.len());
    {
        let spare = buffer.spare_capacity_mut();
        let positions = &mut histograms[0];
        for value in values.iter() {
            let bucket = key(value).radix_byte(0);
            let position = &mut positions[bucket];
            assert!(*position < ends[bucket]);
            spare[*position].write(value.clone());
            *position += 1;
        }
    }
    // Every bucket filled its disjoint range completely.
    unsafe { buffer.set_len(values.len()) };
    for (byte, positions) in histograms.iter_mut().enumerate().skip(1) {
        macro_rules! distribute {
            ($source:expr, $destination:expr) => {{
                let source = $source;
                let destination = $destination;
                for value in source {
                    let bucket = key(value).radix_byte(byte);
                    let position = &mut positions[bucket];
                    destination[*position].clone_from(value);
                    *position += 1;
                }
            }};
        }
        if byte % 2 == 0 {
            distribute!(&*values, &mut buffer);
        } else {
            distribute!(&buffer, &mut *values);
        }
    }
    if K::BYTES % 2 == 1 {
        values.clone_from_slice(&buffer);
    }
}

fn bubble_sort<T, F>(v: &mut [T], mut is_less: F)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();
    if len <= 1 {
        return;
    }
    for i in 0..len - 1 {
        for j in 0..len - i - 1 {
            unsafe {
                if is_less(v.get_unchecked(j + 1), v.get_unchecked(j)) {
                    v.swap(j, j + 1);
                }
            }
        }
    }
}

unsafe fn merge<T, F>(v: &mut [T], mut mid: usize, buf: *mut T, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    unsafe {
        let len = v.len();
        let v = v.as_mut_ptr();
        let (v_mid, v_end) = (v.add(mid), v.add(len));

        copy_nonoverlapping(v, buf, mid);
        let mut start = buf;
        let end = buf.add(mid);
        let mut dest = v;

        let left = &mut start;
        let mut right = v_mid;
        while *left < end && right < v_end {
            let to_copy = if is_less(&*right, &**left) {
                get_and_increment(&mut right)
            } else {
                mid -= 1;
                get_and_increment(left)
            };
            copy_nonoverlapping(to_copy, get_and_increment(&mut dest), 1);
        }

        // let len = end.sub_ptr(start);
        copy_nonoverlapping(start, dest, mid);
    }

    unsafe fn get_and_increment<T>(ptr: &mut *mut T) -> *mut T {
        let old = *ptr;
        *ptr = unsafe { ptr.offset(1) };
        old
    }
}

fn merge_sort<T, F>(v: &mut [T], mut is_less: F)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();
    if len <= 1 {
        return;
    }
    let mut buf = Vec::with_capacity(len / 2);
    let mut runs: Vec<Run> = vec![];
    let mut end = len;
    while end > 0 {
        let start = end - 1;
        let mut left = Run {
            start,
            len: end - start,
        };
        end = start;

        while let Some(right) = runs.pop_if(|right| left.start == 0 || right.len <= left.len) {
            unsafe {
                merge(
                    &mut v[left.start..right.start + right.len],
                    left.len,
                    buf.as_mut_ptr(),
                    &mut is_less,
                );
            }
            left = Run {
                start: left.start,
                len: left.len + right.len,
            };
        }
        runs.push(left);
    }

    debug_assert!(runs.len() == 1 && runs[0].start == 0 && runs[0].len == len);

    #[derive(Clone, Copy)]
    struct Run {
        start: usize,
        len: usize,
    }
}

fn insertion_sort<T, F>(v: &mut [T], mut is_less: F)
where
    F: FnMut(&T, &T) -> bool,
{
    for i in 1..v.len() {
        let x = &v[i];
        let p = v[..i].partition_point(|y| is_less(y, x));
        v[p..=i].rotate_right(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{algorithm::SliceCombinationsExt, tools::Xorshift};

    macro_rules! test_sort {
        (@small $sort_method:ident) => {
            for n in 0..=8 {
                let a: Vec<_> = (0..n).collect();
                a.for_each_permutations(n, |a| {
                    let mut x = a.to_vec();
                    let mut y = a.to_vec();
                    x.sort();
                    y.$sort_method();
                    assert_eq!(x, y);
                });
            }
        };
        (@large $sort_method:ident, $n_ub:expr) => {{
            let mut rng = Xorshift::default();
            for _ in 0..10 {
                let n = rng.random(..$n_ub);
                let ub = 1 << rng.random(0..20);
                let a: Vec<_> = rng.random_iter(0..ub).take(n).collect();
                let mut x = a.to_vec();
                let mut y = a.to_vec();
                x.sort();
                y.$sort_method();
                assert_eq!(x, y);
            }
        }};
    }

    #[test]
    fn test_bubble_sort_small() {
        test_sort!(@small bubble_sort);
    }

    #[test]
    fn test_bubble_sort_large() {
        test_sort!(@large bubble_sort, 3000);
    }

    #[test]
    fn test_merge_sort_small() {
        test_sort!(@small merge_sort);
    }

    #[test]
    fn test_merge_sort_large() {
        test_sort!(@large merge_sort, 100_000);
    }

    #[test]
    fn test_insertion_sort_small() {
        test_sort!(@small insertion_sort);
    }

    #[test]
    fn test_insertion_sort_large() {
        test_sort!(@large insertion_sort, 100_000);
    }

    #[test]
    fn test_radix_sort() {
        let mut rng = Xorshift::default();
        macro_rules! test_types {
            ($($t:ty),* $(,)?) => {
                $(
                    for _ in 0..20 {
                        let n = rng.random(0..1000);
                        let mut actual: Vec<($t, usize)> =
                            rng.random_iter(..).take(n).zip(0..).collect();
                        let mut expected = actual.clone();
                        actual.radix_sort_by_key(|&(key, _)| key);
                        expected.sort_by_key(|&(key, _)| key);
                        assert_eq!(actual, expected);
                    }
                )*
            };
        }
        test_types!(u8, u16, u32, u64, u128, usize);
        test_types!(i8, i16, i32, i64, i128, isize);
        for _ in 0..20 {
            let n = rng.random(0..1000);
            let mut actual: Vec<_> = rng
                .random_iter(0u32..64)
                .take(n)
                .zip(0..)
                .map(|(key, index)| (key, index.to_string()))
                .collect();
            let mut expected = actual.clone();
            actual.radix_sort_by_key(|value| value.0);
            expected.sort_by_key(|value| value.0);
            assert_eq!(actual, expected);
        }
    }
}
