use std::cmp::Ordering;

/// binary search helper
pub trait Bisect: Clone {
    /// Return between two elements if search is not end.
    fn middle_point(&self, other: &Self) -> Option<Self>;
}

macro_rules! impl_bisect_unsigned {
    ($($t:ty)*) => {
        $(impl Bisect for $t {
            fn middle_point(&self, other: &Self) -> Option<Self> {
                let (diff, small) = if self > other { (self - other, other) } else { (other - self, self) };
                if diff > 1 { Some(small + diff / 2) } else { None }
            }
        })*
    };
}
macro_rules! impl_bisect_signed {
    ($($t:ty)*) => {
        $(impl Bisect for $t {
            fn middle_point(&self, other: &Self) -> Option<Self> {
                if self.signum() != other.signum() {
                    if match self.cmp(other) {
                        Ordering::Less => self + 1 < *other,
                        Ordering::Equal => false,
                        Ordering::Greater => other + 1 < *self,
                    } {
                        Some((self + other) / 2)
                    } else {
                        None
                    }
                } else {
                    let (diff, small) = if self > other { (self - other, other) } else { (other - self, self) };
                    if diff > 1 { Some(small + diff / 2) } else { None }
                }
            }
        })*
    };
}
macro_rules! impl_bisect_float {
    ($({$t:ident $u:ident $i:ident $e:expr})*) => {
        $(impl Bisect for $t {
            fn middle_point(&self, other: &Self) -> Option<Self> {
                fn to_float_ord(x: $t) -> $i {
                    let a = x.to_bits() as $i;
                    a ^ (((a >> $e) as $u) >> 1) as $i
                }
                fn from_float_ord(a: $i) -> $t {
                    $t::from_bits((a ^ (((a >> $e) as $u) >> 1) as $i) as _)
                }
                <$i as Bisect>::middle_point(&to_float_ord(*self), &to_float_ord(*other)).map(from_float_ord)
            }
        })*
    };
}
impl_bisect_unsigned!(u8 u16 u32 u64 u128 usize);
impl_bisect_signed!(i8 i16 i32 i64 i128 isize);
impl_bisect_float!({f32 u32 i32 31} {f64 u64 i64 63});

/// binary search for monotone segment
///
/// if `ok < err` then search [ok, err) where t(`ok`), t, t, .... t, t(`ret`), f,  ... f, f, f, `err`
///
/// if `err < ok` then search (err, ok] where `err`, f, f, f, ... f, t(`ret`), ... t, t, t(`ok`)
pub fn binary_search<T, F>(mut f: F, mut ok: T, mut err: T) -> T
where
    T: Bisect,
    F: FnMut(&T) -> bool,
{
    while let Some(m) = ok.middle_point(&err) {
        if f(&m) {
            ok = m;
        } else {
            err = m;
        }
    }
    ok
}

/// binary search for slice
pub trait SliceBisectExt<T> {
    /// Returns the first element that satisfies a predicate.
    fn find_bisect(&self, f: impl FnMut(&T) -> bool) -> Option<&T>;
    /// Returns the last element that satisfies a predicate.
    fn rfind_bisect(&self, f: impl FnMut(&T) -> bool) -> Option<&T>;
    /// Returns the first index that satisfies a predicate.
    /// if not found, returns `len()`.
    fn position_bisect(&self, f: impl FnMut(&T) -> bool) -> usize;
    /// Returns the last index+1 that satisfies a predicate.
    /// if not found, returns `0`.
    fn rposition_bisect(&self, f: impl FnMut(&T) -> bool) -> usize;
}
impl<T> SliceBisectExt<T> for [T] {
    fn find_bisect(&self, f: impl FnMut(&T) -> bool) -> Option<&T> {
        self.get(self.position_bisect(f))
    }
    fn rfind_bisect(&self, f: impl FnMut(&T) -> bool) -> Option<&T> {
        let pos = self.rposition_bisect(f);
        if pos == 0 { None } else { self.get(pos - 1) }
    }
    fn position_bisect(&self, mut f: impl FnMut(&T) -> bool) -> usize {
        binary_search(|i| f(&self[*i as usize]), self.len() as i64, -1) as usize
    }
    fn rposition_bisect(&self, mut f: impl FnMut(&T) -> bool) -> usize {
        binary_search(|i| f(&self[i - 1]), 0, self.len() + 1)
    }
}

pub fn parallel_binary_search<T, F, G>(mut f: F, q: usize, ok: T, err: T) -> Vec<T>
where
    T: Bisect,
    F: FnMut(&[Option<T>]) -> G,
    G: Fn(usize) -> bool,
{
    let mut ok = vec![ok; q];
    let mut err = vec![err; q];
    loop {
        let m: Vec<_> = ok
            .iter()
            .zip(&err)
            .map(|(ok, err)| ok.middle_point(err))
            .collect();
        if m.iter().all(|m| m.is_none()) {
            break;
        }
        let g = f(&m);
        for (i, m) in m.into_iter().enumerate() {
            if let Some(m) = m {
                if g(i) {
                    ok[i] = m;
                } else {
                    err[i] = m;
                }
            }
        }
    }
    ok
}

#[cfg(test)]
mod tests {
    use super::*;
    const V: [i64; 10] = [0i64, 1, 1, 1, 2, 2, 3, 4, 7, 8];

    #[test]
    fn test_binary_search() {
        assert_eq!(binary_search(|&x| V[x] >= 1, V.len(), 0), 1);
        assert_eq!(binary_search(|&x| V[x] >= 2, V.len(), 0), 4);
        assert_eq!(binary_search(|&x| V[x] >= 3, V.len(), 0), 6);
        assert_eq!(binary_search(|&x| V[x] <= 1, 0, V.len()), 3);
        assert_eq!(binary_search(|&x| V[x] <= 2, 0, V.len()), 5);
        assert_eq!(binary_search(|&x| V[x] <= 3, 0, V.len()), 6);

        assert_eq!(
            binary_search(&|&x: &i64| V[x as usize] <= -1, -1, V.len() as i64),
            -1
        );

        let sq2 = binary_search(|&x| x * x <= 2., 1., 4.);
        let expect = 1.414_213_562_73;
        assert!(expect - 1e-8 <= sq2 && sq2 <= expect + 1e-8);

        assert_eq!(
            binary_search(|&x| x < i64::MAX, i64::MIN, i64::MAX),
            i64::MAX - 1
        );
        assert_eq!(
            binary_search(|&x| x == i64::MIN, i64::MIN, i64::MAX),
            i64::MIN
        );
        assert_eq!(
            binary_search(|&x| x == i64::MAX, i64::MAX, i64::MIN),
            i64::MAX
        );
        assert_eq!(
            binary_search(|&x| x > i64::MIN, i64::MAX, i64::MIN),
            i64::MIN + 1
        );
    }

    #[test]
    fn test_position() {
        assert_eq!(V.position_bisect(|&x| x >= -1), 0);
        assert_eq!(V.position_bisect(|&x| x >= 0), 0);
        assert_eq!(V.position_bisect(|&x| x >= 1), 1);
        assert_eq!(V.position_bisect(|&x| x >= 2), 4);
        assert_eq!(V.position_bisect(|&x| x >= 3), 6);
        assert_eq!(V.position_bisect(|&x| x >= 5), 8);
        assert_eq!(V.position_bisect(|&x| x >= 10), 10);
    }

    #[test]
    fn test_find() {
        assert_eq!(V.find_bisect(|&x| x >= -1), Some(&0));
        assert_eq!(V.find_bisect(|&x| x >= 0), Some(&0));
        assert_eq!(V.find_bisect(|&x| x >= 1), Some(&1));
        assert_eq!(V.find_bisect(|&x| x >= 2), Some(&2));
        assert_eq!(V.find_bisect(|&x| x >= 3), Some(&3));
        assert_eq!(V.find_bisect(|&x| x >= 5), Some(&7));
        assert_eq!(V.find_bisect(|&x| x >= 10), None);
    }

    #[test]
    fn test_rposition() {
        assert_eq!(V.rposition_bisect(|&x| x <= -1), 0);
        assert_eq!(V.rposition_bisect(|&x| x <= 0), 1);
        assert_eq!(V.rposition_bisect(|&x| x <= 1), 4);
        assert_eq!(V.rposition_bisect(|&x| x <= 2), 6);
        assert_eq!(V.rposition_bisect(|&x| x <= 3), 7);
        assert_eq!(V.rposition_bisect(|&x| x <= 5), 8);
        assert_eq!(V.rposition_bisect(|&x| x <= 10), 10);
    }

    #[test]
    fn test_rfind() {
        assert_eq!(V.rfind_bisect(|&x| x <= -1), None);
        assert_eq!(V.rfind_bisect(|&x| x <= 0), Some(&0));
        assert_eq!(V.rfind_bisect(|&x| x <= 1), Some(&1));
        assert_eq!(V.rfind_bisect(|&x| x <= 2), Some(&2));
        assert_eq!(V.rfind_bisect(|&x| x <= 3), Some(&3));
        assert_eq!(V.rfind_bisect(|&x| x <= 5), Some(&4));
        assert_eq!(V.rfind_bisect(|&x| x <= 10), Some(&8));
    }
}
