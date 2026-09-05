use std::ops::Range;

pub struct Static2DTree<T, U, V>
where
    T: Ord,
    U: Ord,
{
    data: Vec<(T, U, V)>,
}
impl<T, U, V> Static2DTree<T, U, V>
where
    T: Ord,
    U: Ord,
{
    pub fn new<I>(data: I) -> Self
    where
        I: IntoIterator<Item = (T, U, V)>,
    {
        let mut data: Vec<_> = data.into_iter().collect();
        let n = data.len();
        Self::build(&mut data, 0, n, 0);
        Self { data }
    }
    fn build(data: &mut [(T, U, V)], l: usize, r: usize, depth: usize) {
        if r - l <= 1 {
            return;
        }
        let m = l.midpoint(r);
        if depth.is_multiple_of(2) {
            data[l..r].select_nth_unstable_by(m - l, |p, q| p.0.cmp(&q.0));
        } else {
            data[l..r].select_nth_unstable_by(m - l, |p, q| p.1.cmp(&q.1));
        }
        Self::build(data, l, m, depth + 1);
        Self::build(data, m + 1, r, depth + 1);
    }
    /// Returns the values in the half-open rectangle. Their order is unspecified.
    pub fn range(&self, range1: Range<T>, range2: Range<U>) -> Vec<&V> {
        let mut res = vec![];
        self.range_inner(&range1, &range2, 0, self.data.len(), 0, &mut res);
        res
    }
    fn range_inner<'a>(
        &'a self,
        range1: &Range<T>,
        range2: &Range<U>,
        l: usize,
        r: usize,
        depth: usize,
        res: &mut Vec<&'a V>,
    ) {
        if l < r {
            let m = l.midpoint(r);
            let (t, u, v) = &self.data[m];
            if range1.contains(t) && range2.contains(u) {
                res.push(v);
            }
            if if depth.is_multiple_of(2) {
                &range1.start <= t
            } else {
                &range2.start <= u
            } {
                self.range_inner(range1, range2, l, m, depth + 1, res);
            }
            if if depth.is_multiple_of(2) {
                t < &range1.end
            } else {
                u < &range2.end
            } {
                self.range_inner(range1, range2, m + 1, r, depth + 1, res);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    #[test]
    fn test_static_2d_tree() {
        let mut rng = Xorshift::default();
        for _ in 0..64 {
            let len = rng.rand(300) as usize;
            let radius = rng.rand(64) as i32 + 1;
            let values: Vec<_> = (0..len)
                .map(|index| {
                    (
                        rng.rand((radius * 2 + 1) as u64) as i32 - radius,
                        rng.rand((radius * 2 + 1) as u64) as i32 - radius,
                        index,
                    )
                })
                .collect();
            let tree = Static2DTree::new(values.iter().copied());
            for _ in 0..200 {
                let mut x = [
                    rng.rand((radius * 2 + 3) as u64) as i32 - radius - 1,
                    rng.rand((radius * 2 + 3) as u64) as i32 - radius - 1,
                ];
                let mut y = [
                    rng.rand((radius * 2 + 3) as u64) as i32 - radius - 1,
                    rng.rand((radius * 2 + 3) as u64) as i32 - radius - 1,
                ];
                x.sort_unstable();
                y.sort_unstable();

                let mut actual: Vec<_> = tree
                    .range(x[0]..x[1], y[0]..y[1])
                    .into_iter()
                    .copied()
                    .collect();
                let mut expected: Vec<_> = values
                    .iter()
                    .filter(|(px, py, _)| x[0] <= *px && *px < x[1] && y[0] <= *py && *py < y[1])
                    .map(|(_, _, value)| *value)
                    .collect();
                actual.sort_unstable();
                expected.sort_unstable();
                assert_eq!(actual, expected);
            }
        }
    }
}
