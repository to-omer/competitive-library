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
        if l < r {
            let m = l + (r - l) / 2;
            if depth.is_multiple_of(2) {
                data[l..r].sort_by(|p, q| p.0.cmp(&q.0));
            } else {
                data[l..r].sort_by(|p, q| p.1.cmp(&q.1));
            }
            Self::build(data, l, m, depth + 1);
            Self::build(data, m + 1, r, depth + 1);
        }
    }
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
            let m = l + (r - l) / 2;
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
