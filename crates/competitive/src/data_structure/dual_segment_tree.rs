use super::{MonoidAct, RangeBoundsExt, Unital};
use std::{
    fmt::{self, Debug, Formatter},
    mem::replace,
    ops::RangeBounds,
};

pub struct DualSegmentTree<M>
where
    M: MonoidAct,
{
    n: usize,
    keys: Vec<M::Key>,
    lazy: Vec<M::Act>,
}

impl<M> Clone for DualSegmentTree<M>
where
    M: MonoidAct<Key: Clone>,
{
    fn clone(&self) -> Self {
        Self {
            n: self.n,
            keys: self.keys.clone(),
            lazy: self.lazy.clone(),
        }
    }
}

impl<M> Debug for DualSegmentTree<M>
where
    M: MonoidAct<Key: Debug, Act: Debug>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("DualSegmentTree")
            .field("n", &self.n)
            .field("keys", &self.keys)
            .field("lazy", &self.lazy)
            .finish()
    }
}

impl<M> DualSegmentTree<M>
where
    M: MonoidAct<Key: Clone, Act: PartialEq>,
{
    pub fn from_keys(keys: impl ExactSizeIterator<Item = M::Key>) -> Self {
        let keys: Vec<_> = keys.collect();
        let n = keys.len().next_power_of_two();
        Self {
            n,
            keys,
            lazy: vec![M::unit(); n],
        }
    }
    fn update_at(&mut self, k: usize, a: &M::Act) {
        if k < self.n {
            M::operate_assign(&mut self.lazy[k], a);
        } else {
            M::act_assign(&mut self.keys[k - self.n], a);
        }
    }
    fn propagate_at(&mut self, k: usize) {
        let a = replace(&mut self.lazy[k], M::unit());
        if !M::ActMonoid::is_unit(&a) {
            self.update_at(2 * k, &a);
            self.update_at(2 * k + 1, &a);
        }
    }
    pub fn update<R>(&mut self, range: R, a: M::Act)
    where
        R: RangeBounds<usize>,
    {
        let range = range
            .to_range_bounded(0, self.keys.len())
            .expect("invalid range");
        if range.is_empty() || M::ActMonoid::is_unit(&a) {
            return;
        }
        let mut l = range.start + self.n;
        let mut r = range.end + self.n;
        for i in (1..=self.n.trailing_zeros()).rev() {
            if (l >> i) << i != l {
                self.propagate_at(l >> i);
            }
            if (r >> i) << i != r && ((l >> i) << i == l || l >> i != (r - 1) >> i) {
                self.propagate_at((r - 1) >> i);
            }
        }
        while l < r {
            if l & 1 != 0 {
                self.update_at(l, &a);
                l += 1;
            }
            if r & 1 != 0 {
                r -= 1;
                self.update_at(r, &a);
            }
            l >>= 1;
            r >>= 1;
        }
    }
    pub fn get(&self, k: usize) -> M::Key {
        let mut value = self.keys[k].clone();
        let mut k = (k + self.n) >> 1;
        while k > 0 {
            value = M::act(&value, &self.lazy[k]);
            k >>= 1;
        }
        value
    }
    pub fn set(&mut self, k: usize, value: M::Key) {
        assert!(k < self.keys.len());
        let index = k + self.n;
        for i in (1..=self.n.trailing_zeros()).rev() {
            self.propagate_at(index >> i);
        }
        self.keys[k] = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::LinearAct,
        num::mint_basic::MInt998244353 as M,
        tools::{
            Xorshift,
            testutil::{exhaustive_sequences, sample_usize},
        },
    };

    #[test]
    fn test_dual_segment_tree() {
        let mut rng = Xorshift::default();
        for n in sample_usize(&mut rng, 5, 0..=65, 10) {
            let updates: Vec<_> = (0..=n)
                .flat_map(|l| {
                    (l..=n).flat_map(move |r| {
                        (0..=1).flat_map(move |b| (0..=1).map(move |c| (l, r, b, c)))
                    })
                })
                .collect();
            let sequences: Vec<_> = if n <= 5 {
                exhaustive_sequences(updates, 2..=2).collect()
            } else {
                (0..10)
                    .map(|_| {
                        (0..100)
                            .map(|_| {
                                let l = rng.rand(n as u64 + 1) as usize;
                                let r = l + rng.rand((n - l + 1) as u64) as usize;
                                (l, r, rng.rand(3) as i32, rng.rand(3) as i32)
                            })
                            .collect()
                    })
                    .collect()
            };
            for sequence in sequences {
                let mut values: Vec<_> = (0..n).map(M::from).collect();
                let mut seg = DualSegmentTree::<LinearAct<_>>::from_keys(values.iter().copied());
                for (l, r, b, c) in sequence {
                    let (b, c) = (M::from(b), M::from(c));
                    seg.update(l..r, (b, c));
                    for value in &mut values[l..r] {
                        *value = b * *value + c;
                    }
                    for (i, &value) in values.iter().enumerate() {
                        assert_eq!(seg.get(i), value);
                    }
                }
                for i in 0..n {
                    values[i] = M::from(i);
                    seg.set(i, values[i]);
                    for (j, &value) in values.iter().enumerate() {
                        assert_eq!(seg.get(j), value);
                    }
                }
            }
        }
    }
}
