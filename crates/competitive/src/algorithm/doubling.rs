use super::Monoid;

pub struct Doubling<M>
where
    M: Monoid,
{
    size: usize,
    table: Vec<(usize, M::T)>,
}

impl<M> Doubling<M>
where
    M: Monoid,
{
    pub fn new(size: usize, f: impl Fn(usize) -> (usize, M::T)) -> Self {
        let mut table = Vec::with_capacity(size);
        for i in 0..size {
            table.push(f(i));
        }
        Self { size, table }
    }

    pub fn double(&mut self) {
        let base = self.table.len() - self.size;
        for i in 0..self.size {
            let &(to, ref val) = &self.table[base + i];
            if to != !0 {
                let &(to2, ref val2) = &self.table[base + to];
                self.table.push((to2, M::operate(val, val2)));
            } else {
                self.table.push((!0, M::unit()));
            }
        }
    }

    pub fn kth(&mut self, mut pos: usize, mut k: usize) -> (usize, M::T) {
        let mut x = M::unit();
        for chunk in self.table.chunks_exact(self.size) {
            if k & 1 == 1 {
                let &(to, ref val) = &chunk[pos];
                if to == !0 {
                    return (!0, M::unit());
                }
                x = M::operate(&x, val);
                pos = to;
            }
            k >>= 1;
            if k == 0 {
                break;
            }
        }
        while k > 0 {
            self.double();
            if k & 1 == 1 {
                let base = self.table.len() - self.size;
                let &(to, ref val) = &self.table[base + pos];
                if to == !0 {
                    return (!0, M::unit());
                }
                x = M::operate(&x, val);
                pos = to;
            }
            k >>= 1;
        }
        (pos, x)
    }

    /// queries: (pos, k)
    /// Return: (pos, acc)
    pub fn kth_multiple(
        &self,
        queries: impl IntoIterator<Item = (usize, usize)>,
    ) -> Vec<(usize, M::T)> {
        let (mut ks, mut results): (Vec<usize>, Vec<(usize, M::T)>) = queries
            .into_iter()
            .map(|(start, k)| (k, (start, M::unit())))
            .unzip();
        for chunk in self.table.chunks_exact(self.size) {
            for (i, k) in ks.iter_mut().enumerate() {
                if *k & 1 == 1 {
                    let &(to, ref val) = &chunk[results[i].0];
                    if to == !0 {
                        results[i] = (!0, M::unit());
                        *k = 0;
                    } else {
                        results[i].1 = M::operate(&results[i].1, val);
                        results[i].0 = to;
                    }
                }
                *k >>= 1;
            }
        }
        if ks.iter().any(|&k| k > 0) {
            let mut dp = self.table[self.table.len() - self.size..].to_vec();
            while ks.iter().any(|&k| k > 0) {
                let mut ndp = Vec::with_capacity(dp.len());
                for i in 0..self.size {
                    let &(to, ref val) = &dp[i];
                    if to != !0 {
                        let &(to2, ref val2) = &dp[to];
                        ndp.push((to2, M::operate(val, val2)));
                    } else {
                        ndp.push((!0, M::unit()));
                    }
                }
                dp = ndp;
                for (i, k) in ks.iter_mut().enumerate() {
                    if *k & 1 == 1 {
                        let &(to, ref val) = &dp[results[i].0];
                        if to == !0 {
                            results[i] = (!0, M::unit());
                            *k = 0;
                        } else {
                            results[i].1 = M::operate(&results[i].1, val);
                            results[i].0 = to;
                        }
                    }
                    *k >>= 1;
                }
            }
        }
        results
    }

    /// Return: (k, (pos, acc))
    pub fn find_last(
        &self,
        mut pos: usize,
        mut pred: impl FnMut(usize, &M::T) -> bool,
    ) -> (usize, (usize, M::T)) {
        let mut k = 0usize;
        let mut x = M::unit();
        assert!(pred(pos, &x));
        for (i, chunk) in self.table.chunks_exact(self.size).enumerate().rev() {
            let &(to, ref val) = &chunk[pos];
            let nx = M::operate(&x, val);
            if pred(to, &nx) {
                x = nx;
                pos = to;
                k |= 1 << i;
            }
        }
        (k, (pos, x))
    }

    /// Return: (k, (pos, acc))
    pub fn find_first(
        &self,
        pos: usize,
        mut pred: impl FnMut(usize, &M::T) -> bool,
    ) -> Option<(usize, (usize, M::T))> {
        let (mut k, (mut pos, mut x)) = self.find_last(pos, |k, x| !pred(k, x));
        k += 1;
        M::operate_assign(&mut x, &self.table[pos].1);
        pos = self.table[pos].0;
        if pred(pos, &x) {
            Some((k, (pos, x)))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::AdditiveOperation,
        num::{Zero as _, mint_basic::MInt998244353},
        tools::Xorshift,
    };

    #[test]
    fn test_kth() {
        let mut rng = Xorshift::default();
        for _ in 0..200 {
            let n = rng.random(1usize..100);
            let to: Vec<_> = rng
                .random_iter(0..=n)
                .take(n)
                .map(|x| x.wrapping_sub(1))
                .collect();
            let w: Vec<MInt998244353> = rng.random_iter(..).take(n).collect();
            let mut doubling = Doubling::<AdditiveOperation<_>>::new(n, |i| (to[i], w[i]));
            let mut queries = vec![];
            let mut results = vec![];
            for s in 0..n {
                let mut pos = s;
                let mut x = MInt998244353::zero();
                for k in 0..100 {
                    if pos == !0 {
                        assert_eq!(doubling.kth(s, k), (pos, MInt998244353::zero()));
                        queries.push((s, k));
                        results.push((pos, MInt998244353::zero()));
                    } else {
                        assert_eq!(doubling.kth(s, k), (pos, x));
                        x += w[pos];
                        pos = to[pos];
                    }
                }
            }
            let doubling = Doubling::<AdditiveOperation<_>>::new(n, |i| (to[i], w[i]));
            assert_eq!(doubling.kth_multiple(queries), results);
        }
    }

    #[test]
    fn test_find() {
        let mut rng = Xorshift::default();
        for _ in 0..200 {
            let n = rng.random(1usize..100);
            let to: Vec<_> = rng.random_iter(0..n).take(n).collect();
            let w: Vec<u64> = rng.random_iter(1..100).take(n).collect();
            let mut doubling = Doubling::<AdditiveOperation<_>>::new(n, |i| (to[i], w[i]));
            for _ in 0..10 {
                doubling.double();
            }
            for s in 0..n {
                let mut k = 0usize;
                let mut pos = s;
                let mut acc = 0u64;
                for x in 0u64..200 {
                    while acc + w[pos] <= x {
                        acc += w[pos];
                        pos = to[pos];
                        k += 1;
                    }
                    assert_eq!(doubling.find_last(s, |_, &v| v <= x), (k, (pos, acc)));
                    assert_eq!(
                        doubling.find_first(s, |_, &v| v > x),
                        Some((k + 1, (to[pos], acc + w[pos])))
                    );
                }
                assert_eq!(doubling.find_first(s, |_, &v| v > 1_000_000), None);
            }
        }
    }
}
