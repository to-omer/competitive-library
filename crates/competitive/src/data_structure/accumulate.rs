#[codesnip::skip]
use crate::algebra::{Group, Monoid};

#[derive(Debug, Clone)]
/// Accumlated data
pub struct Accumulate<M: Monoid> {
    data: Vec<M::T>,
}
impl<M: Monoid> std::iter::FromIterator<M::T> for Accumulate<M> {
    fn from_iter<T: IntoIterator<Item = M::T>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let (lower, _) = iter.size_hint();
        let mut data = Vec::with_capacity(lower.saturating_add(1));
        let mut acc = M::unit();
        for x in iter {
            let y = M::operate(&acc, &x);
            data.push(acc);
            acc = y;
        }
        data.push(acc);
        Self { data }
    }
}
impl<M: Monoid> Accumulate<M> {
    /// Return accumlate of \[0, k\)
    pub fn accumulate(&self, k: usize) -> M::T {
        assert!(
            k < self.data.len(),
            "index out of range: the len is {} but the index is {}",
            self.data.len(),
            k
        );
        unsafe { self.data.get_unchecked(k) }.clone()
    }
}
impl<G: Group> Accumulate<G> {
    /// Return fold of \[l, r\)
    pub fn fold(&self, l: usize, r: usize) -> G::T {
        assert!(l <= r, "bad range [{}, {})", l, r);
        assert!(
            r < self.data.len(),
            "index out of range: the len is {} but the index is {}",
            self.data.len(),
            r
        );
        G::operate(&G::inverse(unsafe { self.data.get_unchecked(l) }), unsafe {
            self.data.get_unchecked(r)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{LinearOperation, Magma, Unital},
        num::mint_basic::MInt1000000007,
        rand,
        tools::{RandomSpec, Xorshift},
    };
    type M = LinearOperation<MInt1000000007>;
    struct D;
    impl RandomSpec<MInt1000000007> for D {
        fn rand(&self, rng: &mut Xorshift) -> MInt1000000007 {
            MInt1000000007::new_unchecked(rng.gen(..MInt1000000007::get_mod()))
        }
    }

    #[test]
    fn test_accumlate() {
        let mut rng = Xorshift::default();
        const N: usize = 1_000_000;
        rand!(rng, v: [(D, D); N]);
        let acc: Accumulate<M> = v.iter().cloned().collect();
        let mut d = vec![M::unit(); N + 1];
        for i in 0..N {
            d[i + 1] = M::operate(&d[i], &v[i]);
        }
        for (k, d) in d.iter().enumerate() {
            assert_eq!(acc.accumulate(k), *d);
        }
    }

    #[test]
    fn test_fold() {
        let mut rng = Xorshift::default();
        const N: usize = 1_000;
        rand!(rng, v: [(D, D); N]);
        let acc: Accumulate<M> = v.iter().cloned().collect();
        for l in 0..=N {
            let mut d = M::unit();
            for (r, v) in v.iter().enumerate().skip(l) {
                assert_eq!(acc.fold(l, r), d);
                M::operate_assign(&mut d, &v);
            }
            assert_eq!(acc.fold(l, N), d);
        }
    }
}
