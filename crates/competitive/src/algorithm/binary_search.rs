use std::cmp::Ordering;

/// binary search helper
pub trait Bisect: Clone {
    /// Return between two elements if search is not end.
    fn bisect_middle_point(&self, other: &Self) -> Option<Self>;
}

macro_rules! impl_bisect_unsigned {
    ($($t:ty)*) => {
        $(impl Bisect for $t {
            fn bisect_middle_point(&self, other: &Self) -> Option<Self> {
                if self.abs_diff(*other) > 1 { Some(self.midpoint(*other)) } else { None }
            }
        })*
    };
}
macro_rules! impl_bisect_signed {
    ($($t:ty)*) => {
        $(impl Bisect for $t {
            fn bisect_middle_point(&self, other: &Self) -> Option<Self> {
                if self.signum() != other.signum() {
                    if match self.cmp(other) {
                        Ordering::Less => self + 1 < *other,
                        Ordering::Equal => false,
                        Ordering::Greater => other + 1 < *self,
                    } {
                        Some((*self).midpoint(*other))
                    } else {
                        None
                    }
                } else {
                    if self.abs_diff(*other) > 1 { Some(self.midpoint(*other)) } else { None }
                }
            }
        })*
    };
}
macro_rules! impl_bisect_float {
    ($({$t:ident $u:ident $i:ident $e:expr})*) => {
        $(impl Bisect for $t {
            fn bisect_middle_point(&self, other: &Self) -> Option<Self> {
                fn to_float_ord(x: $t) -> $i {
                    let a = x.to_bits() as $i;
                    a ^ (((a >> $e) as $u) >> 1) as $i
                }
                fn from_float_ord(a: $i) -> $t {
                    $t::from_bits((a ^ (((a >> $e) as $u) >> 1) as $i) as _)
                }
                <$i as Bisect>::bisect_middle_point(&to_float_ord(*self), &to_float_ord(*other)).map(from_float_ord)
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
    while let Some(m) = ok.bisect_middle_point(&err) {
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
            .map(|(ok, err)| ok.bisect_middle_point(err))
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
    use crate::algorithm::SliceCombinationsExt;
    use crate::tools::{
        Xorshift,
        testutil::{integer_boundary_values, sample_usize, structured_sequences},
    };

    #[test]
    fn test_slice_bisect() {
        let mut rng = Xorshift::default();
        let mut cases = Vec::new();
        // Every sorted sequence over {-1, 0, 1}, without enumerating its permutations.
        for n in 0..=32 {
            [-1, 0, 1].for_each_combinations_with_replacement(n, |xs| cases.push(xs.to_vec()));
        }
        let lengths = sample_usize(&mut rng, 32, 0..=1024, 1000);
        cases.extend(
            structured_sequences(&mut rng, -10..=10, lengths).map(|mut values| {
                values.sort_unstable();
                values
            }),
        );
        cases.sort_unstable();
        cases.dedup();
        for values in cases {
            let n = values.len();
            let mut first = 0;
            let mut end = 0;
            for key in -11..=11 {
                while first < n && values[first] < key {
                    first += 1;
                }
                while end < n && values[end] <= key {
                    end += 1;
                }
                assert_eq!(
                    values.position_bisect(|&x| x >= key),
                    first,
                    "values={values:?}, key={key}"
                );
                assert_eq!(
                    values.find_bisect(|&x| x >= key),
                    values.get(first),
                    "values={values:?}, key={key}"
                );
                assert_eq!(
                    values.rposition_bisect(|&x| x <= key),
                    end,
                    "values={values:?}, key={key}"
                );
                assert_eq!(
                    values.rfind_bisect(|&x| x <= key),
                    values[..end].last(),
                    "values={values:?}, key={key}"
                );
                assert_eq!(
                    binary_search(|&i: &isize| values[i as usize] >= key, n as isize, -1),
                    first as isize,
                    "values={values:?}, key={key}"
                );
                assert_eq!(
                    binary_search(|&i: &isize| values[i as usize] <= key, -1, n as isize),
                    end as isize - 1,
                    "values={values:?}, key={key}"
                );
            }
        }
    }

    #[test]
    fn test_integer_bisect() {
        macro_rules! check {
            ($($ty:ty),*) => {$(
                let mut rng = Xorshift::default();
                for boundary in integer_boundary_values!($ty).into_iter()
                    .chain((0..=u8::MAX).map(|x| x as $ty))
                    .chain(rng.random_iter(..).take(10_000))
                {
                    if boundary < <$ty>::MAX {
                        assert_eq!(binary_search(|&x| x <= boundary, <$ty>::MIN, <$ty>::MAX), boundary);
                        assert_eq!(binary_search(|&x| x > boundary, <$ty>::MAX, <$ty>::MIN), boundary + 1);
                    }
                    if boundary > <$ty>::MIN {
                        assert_eq!(binary_search(|&x| x >= boundary, <$ty>::MAX, <$ty>::MIN), boundary);
                        assert_eq!(binary_search(|&x| x < boundary, <$ty>::MIN, <$ty>::MAX), boundary - 1);
                    }
                }
            )*};
        }
        check!(
            u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
        );
    }

    #[test]
    fn test_float_bisect() {
        macro_rules! check {
            ($ty:ty, $bits:ty, $fraction:expr, $max_exponent:expr) => {
                let mut rng = Xorshift::default();
                // Powers of two and their adjacent representations cross every
                // normal/subnormal exponent boundary, in both directions.
                let mut bits: Vec<$bits> = (1..=$max_exponent)
                    .flat_map(|exponent| {
                        let power = exponent << $fraction;
                        [power - 1, power, power + 1]
                    })
                    .collect();
                bits.extend(integer_boundary_values!($bits));
                bits.extend((0..10_000).map(|_| {
                    let bits: $bits = rng.random(..);
                    bits
                }));
                for bits in bits {
                    for x in [<$ty>::from_bits(bits), -<$ty>::from_bits(bits)] {
                        if x.is_finite() && x != 0.0 {
                            assert_eq!(
                                binary_search(|&y| y <= x, <$ty>::NEG_INFINITY, <$ty>::INFINITY),
                                x
                            );
                            assert_eq!(
                                binary_search(|&y| y >= x, <$ty>::INFINITY, <$ty>::NEG_INFINITY),
                                x
                            );
                        }
                    }
                }
                for x in 0..=10_000 {
                    let x = x as $ty;
                    let actual = binary_search(|&y| y * y <= x, 0.0, x + 1.0);
                    assert!((actual - x.sqrt()).abs() <= <$ty>::EPSILON * x.sqrt().max(1.0));
                }
            };
        }
        check!(f32, u32, 23, 254);
        check!(f64, u64, 52, 2046);
    }
}
