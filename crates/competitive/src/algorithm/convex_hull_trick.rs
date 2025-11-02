use std::{
    collections::VecDeque,
    ops::{Add, Mul, Sub},
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChtLine<T> {
    slope: T,
    intercept: T,
}

impl<T> ChtLine<T>
where
    T: Copy + PartialOrd + Add<Output = T> + Sub<Output = T> + Mul<Output = T>,
{
    pub fn new(a: T, b: T) -> Self {
        Self {
            slope: a,
            intercept: b,
        }
    }
    pub fn value(&self, x: T) -> T {
        self.slope * x + self.intercept
    }
    pub fn check(&self, l1: &Self, l2: &Self) -> bool {
        (l1.slope - self.slope) * (l2.intercept - l1.intercept)
            >= (l1.intercept - self.intercept) * (l2.slope - l1.slope)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChtQueryMode {
    Increasing,
    Decreasing,
    Any,
}

#[derive(Debug, Clone)]
pub struct ConvexHullTrick<T> {
    deq: VecDeque<ChtLine<T>>,
    query_mode: ChtQueryMode,
}

impl<T> Default for ConvexHullTrick<T> {
    fn default() -> Self {
        Self {
            deq: Default::default(),
            query_mode: ChtQueryMode::Increasing,
        }
    }
}

impl<T> ConvexHullTrick<T>
where
    T: Copy + PartialOrd + Add<Output = T> + Sub<Output = T> + Mul<Output = T>,
{
    pub fn with_query_increasing() -> Self {
        Self {
            deq: Default::default(),
            query_mode: ChtQueryMode::Increasing,
        }
    }

    pub fn with_query_decreasing() -> Self {
        Self {
            deq: Default::default(),
            query_mode: ChtQueryMode::Decreasing,
        }
    }

    pub fn with_query_any() -> Self {
        Self {
            deq: Default::default(),
            query_mode: ChtQueryMode::Any,
        }
    }

    pub fn add_line(&mut self, a: T, b: T) {
        if self.deq.is_empty() {
            self.deq.push_back(ChtLine::new(a, b));
        } else if let Some(l) = self.deq.back()
            && l.slope >= a
        {
            let line = ChtLine::new(a, b);
            while {
                let k = self.deq.len();
                k > 1 && self.deq[k - 2].check(&self.deq[k - 1], &line)
            } {
                self.deq.pop_back();
            }
            self.deq.push_back(line);
        } else if let Some(l) = self.deq.front()
            && l.slope <= a
        {
            let line = ChtLine::new(a, b);
            while {
                let k = self.deq.len();
                k > 1 && line.check(&self.deq[0], &self.deq[1])
            } {
                self.deq.pop_front();
            }
            self.deq.push_front(line);
        } else {
            panic!("a must be monotonic");
        }
    }

    pub fn query_min(&mut self, x: T) -> T {
        assert!(!self.deq.is_empty(), "no lines added");
        match self.query_mode {
            ChtQueryMode::Increasing => {
                while {
                    let k = self.deq.len();
                    k > 1 && self.deq[0].value(x) >= self.deq[1].value(x)
                } {
                    self.deq.pop_front();
                }
                self.deq.front().unwrap().value(x)
            }
            ChtQueryMode::Decreasing => {
                while {
                    let k = self.deq.len();
                    k > 1 && self.deq[k - 1].value(x) >= self.deq[k - 2].value(x)
                } {
                    self.deq.pop_back();
                }
                self.deq.back().unwrap().value(x)
            }
            ChtQueryMode::Any => {
                let mut l = 0;
                let mut r = self.deq.len() - 1;
                while l < r {
                    let m = (l + r) / 2;
                    if self.deq[m].value(x) >= self.deq[m + 1].value(x) {
                        l = m + 1;
                    } else {
                        r = m;
                    }
                }
                self.deq[l].value(x)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;
    const A: i64 = 1_000_000_000;

    #[test]
    fn test_convex_hull_trick() {
        let mut rng = Xorshift::default();
        for _ in 0..200 {
            let q = rng.random(1..200);
            let mut lines: Vec<_> = rng.random_iter((-A..=A, -A..=A)).take(q).collect();
            let mut xs: Vec<_> = rng.random_iter(-A..=A).take(q).collect();
            for lty in 0..2 {
                if lty == 0 {
                    lines.sort_unstable_by_key(|&(a, _)| -a);
                } else {
                    lines.sort_unstable_by_key(|&(a, _)| a);
                }
                for qty in 0..3 {
                    let mut cht = if qty == 0 {
                        rng.shuffle(&mut xs);
                        ConvexHullTrick::with_query_any()
                    } else if qty == 1 {
                        xs.sort_unstable_by_key(|&x| -x);
                        ConvexHullTrick::with_query_decreasing()
                    } else {
                        xs.sort_unstable();
                        ConvexHullTrick::with_query_increasing()
                    };
                    for (i, (&(a, b), &x)) in lines.iter().zip(&xs).enumerate() {
                        cht.add_line(a, b);
                        let result = cht.query_min(x);
                        let expected = lines[..=i].iter().map(|&(a, b)| a * x + b).min().unwrap();
                        assert_eq!(result, expected);
                    }
                }
            }
        }
    }
}
