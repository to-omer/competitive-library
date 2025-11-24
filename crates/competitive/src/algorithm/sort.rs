use std::{cmp::Ordering, ptr::copy_nonoverlapping};

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

        while let Some(&right) = runs.last() {
            if left.start > 0 && right.len > left.len {
                break;
            }
            runs.pop().unwrap();
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
}
