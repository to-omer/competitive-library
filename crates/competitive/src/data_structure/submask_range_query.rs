use super::{BitDpExt, Group, Xorshift};
use std::fmt::{self, Debug};

#[derive(Debug, Clone, Copy)]
pub struct SubmaskRangeQuery {
    bit_width: u32,
    mask: [u32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryKind {
    Get,
    Update,
}

impl SubmaskRangeQuery {
    pub fn new(bit_width: u32) -> Self {
        let mut rng = Xorshift::new();
        let mut mask = [0; 3];
        let mut rem: Vec<_> = (0..bit_width).map(|w| w % 3).collect();
        rng.shuffle(&mut rem);
        for (k, r) in rem.into_iter().enumerate() {
            mask[r as usize] |= 1 << k;
        }
        Self { bit_width, mask }
    }

    pub fn new_with_queries(
        queries: impl IntoIterator<Item = (QueryKind, u32)> + ExactSizeIterator + Clone,
    ) -> Self {
        let bit_width = queries
            .clone()
            .into_iter()
            .map(|(_, m)| 32 - m.leading_zeros())
            .max()
            .unwrap_or(0);
        let mut mask = [0; 3];
        let mut cost = vec![1u32; queries.len()];
        for k in 0..bit_width {
            let mut sum = [0u64; 3];
            for ((kind, m), &c) in queries.clone().into_iter().zip(&cost) {
                match kind {
                    QueryKind::Get => {
                        let b = m >> k & 1 == 0;
                        sum[if b { 2 } else { 1 }] += c as u64;
                    }
                    QueryKind::Update => {
                        let b = m >> k & 1 == 0;
                        sum[if b { 0 } else { 2 }] += c as u64;
                    }
                }
            }
            let t = (0..3).min_by_key(|&i| sum[i]).unwrap();
            mask[t] |= 1 << k;
            for ((kind, m), c) in queries.clone().into_iter().zip(&mut cost) {
                match kind {
                    QueryKind::Get => {
                        let b = m >> k & 1 == 0;
                        if t == if b { 2 } else { 1 } {
                            *c <<= 1;
                        }
                    }
                    QueryKind::Update => {
                        let b = m >> k & 1 == 0;
                        if t == if b { 0 } else { 2 } {
                            *c <<= 1;
                        }
                    }
                }
            }
        }
        Self { bit_width, mask }
    }

    pub fn builder<G>() -> SubmaskRangeQueryBuilder<G>
    where
        G: Group,
    {
        SubmaskRangeQueryBuilder::new()
    }

    pub fn get_query(&self, m: u32) -> impl Iterator<Item = (u32, bool)> {
        let fix = m & self.mask[0];
        let sub = m & self.mask[1];
        let sup = (!m) & self.mask[2];
        sup.subsets().flat_map(move |s| {
            let inv = s.count_ones() & 1 == 1;
            sub.subsets().map(move |t| (fix | s | t, inv))
        })
    }

    pub fn get_query_nested(
        &self,
        m: u32,
    ) -> impl Iterator<Item = (impl Iterator<Item = u32>, bool)> {
        let fix = m & self.mask[0];
        let sub = m & self.mask[1];
        let sup = (!m) & self.mask[2];
        sup.subsets().map(move |s| {
            let inv = s.count_ones() & 1 == 1;
            let it = sub.subsets().map(move |t| fix | s | t);
            (it, inv)
        })
    }

    pub fn update_query(&self, m: u32) -> impl Iterator<Item = u32> {
        let fix = m & self.mask[0] | m & self.mask[1];
        let sup = (!m) & self.mask[0];
        let sub = m & self.mask[2];
        sub.subsets()
            .flat_map(move |s| sup.subsets().map(move |t| fix | s | t))
    }
}

#[derive(Debug, Clone)]
enum Query<T> {
    Get { m: u32 },
    Update { m: u32, x: T },
}

pub struct SubmaskRangeQueryBuilder<G>
where
    G: Group,
{
    query: Vec<Query<G::T>>,
}

impl<G> Debug for SubmaskRangeQueryBuilder<G>
where
    G: Group,
    G::T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubmaskRangeQueryBuilder")
            .field("query", &self.query)
            .finish()
    }
}

impl<G> Default for SubmaskRangeQueryBuilder<G>
where
    G: Group,
{
    fn default() -> Self {
        Self {
            query: Default::default(),
        }
    }
}

impl<G> SubmaskRangeQueryBuilder<G>
where
    G: Group,
{
    pub fn new() -> Self {
        Default::default()
    }

    pub fn push_get(&mut self, m: u32) {
        self.query.push(Query::Get { m });
    }

    pub fn push_update(&mut self, m: u32, x: G::T) {
        self.query.push(Query::Update { m, x });
    }

    pub fn solve(self) -> Vec<G::T> {
        let s = SubmaskRangeQuery::new_with_queries(self.query.iter().map(|q| match q {
            Query::Get { m } => (QueryKind::Get, *m),
            Query::Update { m, .. } => (QueryKind::Update, *m),
        }));
        let out_size = self
            .query
            .iter()
            .filter(|q| matches!(q, Query::Get { .. }))
            .count();
        let mut out = Vec::with_capacity(out_size);
        let mut data = vec![G::unit(); 1 << s.bit_width];
        for q in self.query {
            match q {
                Query::Get { m } => {
                    let mut f = G::unit();
                    for (it, inv) in s.get_query_nested(m) {
                        let mut g = G::unit();
                        for k in it {
                            G::operate_assign(&mut g, &data[k as usize]);
                        }
                        if inv {
                            G::rinv_operate_assign(&mut f, &g);
                        } else {
                            G::operate_assign(&mut f, &g);
                        }
                    }
                    out.push(f);
                }
                Query::Update { m, x } => {
                    for k in s.update_query(m) {
                        G::operate_assign(&mut data[k as usize], &x);
                    }
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algebra::AdditiveOperation;

    #[test]
    fn test_submask_range_query() {
        const W: u32 = 16;
        let mut rng = Xorshift::default();
        let mut q = SubmaskRangeQuery::builder::<AdditiveOperation<i32>>();
        let mut a = vec![0; 1 << W];
        let mut exp = vec![];
        for _ in 0..2000 {
            if rng.gen_bool(0.5) {
                let i = rng.rand((1 << W) as _) as u32;
                let x = rng.rand(100) as i32;
                q.push_update(i, x);
                a[i as usize] += x;
            } else {
                let i = rng.rand((1 << W) as _) as u32;
                q.push_get(i);
                let mut x = 0;
                for j in 0..1 << W {
                    if (i & j) == j {
                        x += a[j as usize];
                    }
                }
                exp.push(x);
            }
        }
        let ans = q.solve();
        assert_eq!(ans, exp);
    }

    #[test]
    fn test_submask_range_query_online() {
        const W: u32 = 16;
        let mut rng = Xorshift::default();
        let q = SubmaskRangeQuery::new(W);
        let mut a = vec![0; 1 << W];
        let mut b = vec![0; 1 << W];
        for _ in 0..2000 {
            if rng.gen_bool(0.5) {
                let i = rng.rand((1 << W) as _) as u32;
                let x = rng.rand(100) as i32;
                a[i as usize] += x;
                for j in q.update_query(i) {
                    b[j as usize] += x;
                }
            } else {
                let i = rng.rand((1 << W) as _) as u32;
                let mut x = 0;
                for j in 0..1 << W {
                    if (i & j) == j {
                        x += a[j as usize];
                    }
                }
                let mut y = 0;
                for (j, inv) in q.get_query(i) {
                    if inv {
                        y -= b[j as usize];
                    } else {
                        y += b[j as usize];
                    }
                }
                assert_eq!(x, y);
            }
        }
    }
}
