#[cargo_snippet::snippet("ConvexHullTrick")]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CHTLine {
    slope: i64,
    intercept: i64,
}
#[cargo_snippet::snippet("ConvexHullTrick")]
impl CHTLine {
    pub fn new(a: i64, b: i64) -> Self {
        Self {
            slope: a,
            intercept: b,
        }
    }
    pub fn value(&self, x: i64) -> i64 {
        self.slope * x + self.intercept
    }
    pub fn check(&self, l1: &Self, l2: &Self) -> bool {
        (l1.slope - self.slope) * (l2.intercept - l1.intercept)
            >= (l1.intercept - self.intercept) * (l2.slope - l1.slope)
    }
}
#[cargo_snippet::snippet("ConvexHullTrick")]
#[derive(Clone, Debug, Default)]
pub struct ConvexHullTrick {
    deq: std::collections::VecDeque<CHTLine>,
}
#[cargo_snippet::snippet("ConvexHullTrick")]
impl ConvexHullTrick {
    /// k-th add_line(a_k, b_k): a_k >= a_{k+1}
    pub fn add_line(&mut self, a: i64, b: i64) {
        let line = CHTLine::new(a, b);
        while {
            let k = self.deq.len();
            k > 1 && self.deq[k - 2].check(&self.deq[k - 1], &line)
        } {
            self.deq.pop_back();
        }
        self.deq.push_back(line);
    }
    pub fn query(&mut self, x: i64) -> i64 {
        while {
            let k = self.deq.len();
            k > 1 && self.deq[0].value(x) >= self.deq[1].value(x)
        } {
            self.deq.pop_front();
        }
        self.deq.front().unwrap().value(x)
    }
}

// ConvexHullTrick verify: https://atcoder.jp/contests/dp/submissions/11341451
