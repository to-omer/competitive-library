use super::{
    ArrayOperation, BinaryIndexedTree, Invertible, Ring, SemiRing, SliceSortExt, StaticSearch,
};

pub struct StaticRectangleAddRectangleSum<R>
where
    R: SemiRing,
{
    rectangles: Vec<(u32, u32, u32, u32, R::T)>,
    queries: Vec<(u32, u32, u32, u32)>,
}

impl<R> StaticRectangleAddRectangleSum<R>
where
    R: SemiRing,
{
    /// Rectangles are `(left, bottom, right, top, weight)`.
    pub fn new(rectangles: impl IntoIterator<Item = (u32, u32, u32, u32, R::T)>) -> Self {
        Self {
            rectangles: rectangles.into_iter().collect(),
            queries: Vec::new(),
        }
    }

    /// Returns its index in `execute`'s results.
    pub fn query(&mut self, l: u32, d: u32, r: u32, u: u32) -> usize {
        let index = self.queries.len();
        self.queries.push((l, d, r, u));
        index
    }

    /// Results in query order.
    pub fn execute(self) -> Vec<R::T>
    where
        R: Ring<Additive: Invertible>,
        R::T: From<u32>,
    {
        let n = self.rectangles.len();
        let mut ys: Vec<_> = self
            .rectangles
            .iter()
            .flat_map(|&(_, d, _, u, _)| [d, u])
            .collect();
        ys.radix_sort_by_key(|&y| y);
        ys.dedup();
        let search = StaticSearch::from_sorted(&ys);
        let endpoints: Vec<_> = self
            .rectangles
            .iter()
            .flat_map(|&(_, d, _, u, _)| [d, u])
            .chain(self.queries.iter().flat_map(|&(_, d, _, u)| [d, u]))
            .collect();
        let mut positions = vec![0; endpoints.len()];
        search.lower_bound_batch(&endpoints, &mut positions);
        let mut points: Vec<_> = self
            .rectangles
            .into_iter()
            .zip(positions[..2 * n].as_chunks().0)
            .flat_map(|((l, _, r, _, w), &[d, u])| {
                let d = d as u32;
                let u = u as u32;
                let negative = R::neg(&w);
                [
                    (l, d, w.clone()),
                    (l, u, negative.clone()),
                    (r, d, negative),
                    (r, u, w),
                ]
            })
            .collect();
        points.radix_sort_by_key(|&(x, ..)| x);
        let mut events: Vec<_> = self
            .queries
            .into_iter()
            .zip(positions[2 * n..].as_chunks().0)
            .enumerate()
            .flat_map(|(i, ((l, d, r, u), &[di, ui]))| {
                let di = di as u32;
                let ui = ui as u32;
                [
                    (l, d, u, di, ui, i as u32, false),
                    (r, d, u, di, ui, i as u32, true),
                ]
            })
            .collect();
        events.radix_sort_by_key(|&(x, ..)| x);
        let mut bit = BinaryIndexedTree::<ArrayOperation<R::Additive, 4>>::new(ys.len());
        let mut points = points.into_iter().peekable();
        let mut answers = vec![R::zero(); events.len() / 2];
        for (x, d, u, di, ui, i, add) in events {
            while points.peek().is_some_and(|&(px, ..)| px < x) {
                let (px, py, w) = points.next().unwrap();
                let wx = R::mul(&w, &px.into());
                let wy = R::mul(&w, &ys[py as usize].into());
                let wxy = R::mul(&wx, &ys[py as usize].into());
                bit.update(py as usize, [w, wx, wy, wxy]);
            }
            for (y, yi, add) in [(d, di, !add), (u, ui, add)] {
                let [w, wx, wy, wxy] = bit.accumulate0(yi as usize);
                let value = R::add(
                    &R::sub(
                        &R::mul(&R::sub(&R::mul(&w, &x.into()), &wx), &y.into()),
                        &R::mul(&wy, &x.into()),
                    ),
                    &wxy,
                );
                if add {
                    R::add_assign(&mut answers[i as usize], &value);
                } else {
                    R::sub_assign(&mut answers[i as usize], &value);
                }
            }
        }
        answers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::AddMulOperation, num::mint_basic::MInt998244353 as M,
        tools::testutil::exhaustive_sequences,
    };

    #[test]
    fn test_static_rectangle_add_rectangle_sum() {
        let rectangles: Vec<_> = (0..=2u32)
            .flat_map(|l| {
                (l..=2).flat_map(move |r| {
                    (0..=2u32).flat_map(move |d| (d..=2).map(move |u| (l, d, r, u)))
                })
            })
            .collect();
        let weighted: Vec<_> = rectangles
            .iter()
            .flat_map(|&(l, d, r, u)| (-1..=1i64).map(move |w| (l, d, r, u, w)))
            .collect();
        let boundaries =
            [1, i64::MIN, i64::MAX].map(|weight| vec![(0, 0, u32::MAX, u32::MAX, weight)]);
        let queries: Vec<_> = rectangles
            .iter()
            .copied()
            .chain([(0, 0, u32::MAX, u32::MAX)])
            .collect();
        for input in exhaustive_sequences(weighted.iter().copied(), 0..=2).chain(boundaries) {
            let expected: Vec<i128> = queries
                .iter()
                .map(|&(l, d, r, u)| {
                    input
                        .iter()
                        .map(|&(ll, dd, rr, uu, w)| {
                            r.min(rr).saturating_sub(l.max(ll)) as i128
                                * u.min(uu).saturating_sub(d.max(dd)) as i128
                                * w as i128
                        })
                        .sum()
                })
                .collect();
            let mut integer = StaticRectangleAddRectangleSum::<AddMulOperation<i128>>::new(
                input.iter().map(|&(l, d, r, u, w)| (l, d, r, u, w as i128)),
            );
            let mut modular = StaticRectangleAddRectangleSum::<AddMulOperation<M>>::new(
                input
                    .iter()
                    .map(|&(l, d, r, u, w)| (l, d, r, u, M::from(w))),
            );
            for (index, &(l, d, r, u)) in queries.iter().enumerate() {
                assert_eq!(integer.query(l, d, r, u), index);
                assert_eq!(modular.query(l, d, r, u), index);
            }
            assert_eq!(integer.execute(), expected);
            assert_eq!(
                modular.execute(),
                expected.into_iter().map(M::from).collect::<Vec<_>>()
            );
        }
        assert!(
            StaticRectangleAddRectangleSum::<AddMulOperation<i128>>::new(
                weighted
                    .into_iter()
                    .map(|(l, d, r, u, w)| (l, d, r, u, w as i128)),
            )
            .execute()
            .is_empty()
        );
    }
}
