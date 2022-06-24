use super::{GetDistinctMut, Monoid, SegmentTree, SliceBisectExt};
use std::fmt::{self, Debug, Formatter};

pub struct SegmentTree2D<M, X, Y>
where
    M: Monoid,
{
    xs: Vec<X>,
    ys: Vec<Y>,
    index: Vec<Vec<usize>>,
    segs: Vec<SegmentTree<M>>,
}

impl<M, X, Y> Clone for SegmentTree2D<M, X, Y>
where
    M: Monoid,
    X: Clone,
    Y: Clone,
{
    fn clone(&self) -> Self {
        Self {
            xs: self.xs.clone(),
            ys: self.ys.clone(),
            index: self.index.clone(),
            segs: self.segs.clone(),
        }
    }
}

impl<M, X, Y> Debug for SegmentTree2D<M, X, Y>
where
    M: Monoid,
    M::T: Debug,
    X: Debug,
    Y: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("SegmentTree2D")
            .field("xs", &self.xs)
            .field("ys", &self.ys)
            .field("index", &self.index)
            .field("segs", &self.segs)
            .finish()
    }
}

impl<M, X, Y> SegmentTree2D<M, X, Y>
where
    M: Monoid,
    X: Ord + Clone,
    Y: Ord + Clone,
{
    pub fn new(points: &[(X, Y)]) -> Self {
        let mut xs: Vec<_> = points.iter().map(|(x, _)| x.clone()).collect();
        let mut ys: Vec<_> = points.iter().map(|(_, y)| y.clone()).collect();
        xs.sort();
        xs.dedup();
        ys.sort();
        ys.dedup();
        let n = xs.len();
        let mut index = vec![vec![]; n * 2];
        for (x, y) in points {
            let i = xs.binary_search(x).unwrap();
            let j = ys.binary_search(y).unwrap();
            index[i + n].push(j);
        }
        for idx in index[n..n * 2].iter_mut() {
            idx.sort_unstable();
            idx.dedup();
        }
        for i in (1..n).rev() {
            let (p, l, r) = index.get_distinct_mut((i, i * 2, i * 2 + 1));
            p.extend(l.iter());
            p.extend(r.iter());
            index[i].sort_unstable();
            index[i].dedup();
        }
        let segs = index.iter().map(Vec::len).map(SegmentTree::new).collect();
        Self {
            xs,
            ys,
            index,
            segs,
        }
    }
    pub fn update(&mut self, x: X, y: Y, v: M::T) {
        let mut i = self.xs.binary_search(&x).expect("not exist X key") + self.xs.len();
        let j = self.ys.binary_search(&y).expect("not exist Y key");
        while i > 0 {
            let jj = self.index[i].binary_search(&j).unwrap();
            self.segs[i].update(jj, v.clone());
            i /= 2;
        }
    }
    pub fn fold(&self, xl: X, xr: X, yl: Y, yr: Y) -> M::T {
        let mut il = self.xs.position_bisect(|x| x >= &xl) + self.xs.len();
        let mut ir = self.xs.position_bisect(|x| x >= &xr) + self.xs.len();
        let jl = self.ys.position_bisect(|y| y >= &yl);
        let jr = self.ys.position_bisect(|y| y >= &yr);
        let mut v = M::unit();
        while il < ir {
            if il & 1 != 0 {
                let jjl = self.index[il].position_bisect(|j| j >= &jl);
                let jjr = self.index[il].position_bisect(|j| j >= &jr);
                v = M::operate(&v, &self.segs[il].fold(jjl, jjr));
                il += 1;
            }
            if ir & 1 != 0 {
                ir -= 1;
                let jjl = self.index[ir].position_bisect(|j| j >= &jl);
                let jjr = self.index[ir].position_bisect(|j| j >= &jr);
                v = M::operate(&self.segs[ir].fold(jjl, jjr), &v);
            }
            il /= 2;
            ir /= 2;
        }
        v
    }
}
