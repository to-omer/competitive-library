use super::Bounded;
use std::{
    borrow::Borrow,
    cmp::Ordering,
    collections::BTreeSet,
    fmt::{self, Debug},
    ops::{Add, Div, Mul, Sub},
};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct Slope<T>(T);

impl<T> Debug for Slope<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> PartialOrd for Slope<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0).map(|ord| ord.reverse())
    }
}

impl<T> Ord for Slope<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0).reverse()
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Query<T>(T);

impl<T> Debug for Query<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Default)]
struct Line<T> {
    a: Slope<T>,
    b: T,
    q: Query<T>,
}

impl<T> PartialEq for Line<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.a == other.a
    }
}

impl<T> Eq for Line<T> where T: Eq {}

impl<T> PartialOrd for Line<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.a.partial_cmp(&other.a)
    }
}

impl<T> Ord for Line<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.a.cmp(&other.a)
    }
}

impl<T> Borrow<Slope<T>> for Line<T> {
    fn borrow(&self) -> &Slope<T> {
        &self.a
    }
}

impl<T> Borrow<Query<T>> for Line<T> {
    fn borrow(&self) -> &Query<T> {
        &self.q
    }
}

impl<T> Line<T> {
    fn new(a: T, b: T, q: T) -> Self {
        Self {
            a: Slope(a),
            b,
            q: Query(q),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LineSet<T> {
    map: BTreeSet<Line<T>>,
}

impl<T> Default for LineSet<T>
where
    T: Ord,
{
    fn default() -> Self {
        Self {
            map: Default::default(),
        }
    }
}

impl<T> LineSet<T>
where
    T: Copy + Bounded + Ord + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T>,
{
    pub fn new() -> Self {
        Default::default()
    }
    pub fn insert(&mut self, a: T, b: T) {
        fn f<T>(la: T, lb: T, ra: T, rb: T) -> T
        where
            T: Copy + Ord + Sub<Output = T> + Div<Output = T>,
        {
            debug_assert!(la > ra);
            (rb - lb) / (la - ra)
        }
        let left = self.map.range(..Slope(a)).next_back().cloned();
        let mut right = self.map.range(Slope(a)..).next().cloned();
        match (&left, &right) {
            (_, Some(r)) if a == r.a.0 => {
                if r.b <= b {
                    return;
                }
                self.map.remove(r);
                right = self.map.range(Slope(a)..).next().cloned();
            }
            (Some(l), Some(r)) if f(l.a.0, l.b, a, b) >= f(a, b, r.a.0, r.b) => return,
            _ => {}
        }
        loop {
            let rq = if let Some(r) = right {
                let rq = f(a, b, r.a.0, r.b);
                if rq >= r.q.0 {
                    self.map.remove(&r);
                    right = self.map.range(Slope(a)..).next().cloned();
                    continue;
                }
                rq
            } else {
                Bounded::maximum()
            };
            self.map.insert(Line::new(a, b, rq));
            break;
        }
        if let Some(mut l) = left {
            self.map.remove(&l);
            let mut lq = f(l.a.0, l.b, a, b);
            loop {
                if let Some(ll) = self.map.range(..Slope(a)).next_back().cloned() {
                    self.map.remove(&ll);
                    let llq = f(ll.a.0, ll.b, l.a.0, l.b);
                    if llq >= lq {
                        l = ll;
                        lq = f(l.a.0, l.b, a, b);
                        continue;
                    } else {
                        self.map.insert(Line::new(ll.a.0, ll.b, llq));
                    }
                };
                break;
            }
            self.map.insert(Line::new(l.a.0, l.b, lq));
        }
    }
    pub fn query_min(&self, x: T) -> Option<T> {
        let Line { a, b, .. } = self.map.range(Query(x)..).next().cloned()?;
        Some(a.0 * x + b)
    }
}
