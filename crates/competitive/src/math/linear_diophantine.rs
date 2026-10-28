use super::{ExtendedGcd, RangeBoundsExt, Signed, Unsigned};
use std::ops::RangeInclusive;

/// ax + b
#[derive(Clone, Copy, Debug)]
struct Linear<T>
where
    T: Signed,
{
    a: T,
    b: T,
}

impl<T> Linear<T>
where
    T: Signed,
{
    fn new(a: T, b: T) -> Self {
        Self { a, b }
    }
    fn eval(&self, x: T) -> T {
        self.a * x + self.b
    }
}

/// Solution of ax + by = c
#[derive(Clone, Copy, Debug)]
pub struct LinearDiophantineSolution<T>
where
    T: Signed,
{
    x: Linear<T>,
    y: Linear<T>,
    k_range: (T, T),
}

macro_rules! with_range {
    ($this:ident, $x:ident, $range:expr) => {
        let range = $range.to_range_inclusive();
        let l = *range.start();
        let r = *range.end();
        // l <= a * k + b <= r
        if !l.is_minimum() {
            if $this.$x.a.is_positive() {
                // (l - b) / a <= k
                let t = (l - $this.$x.b + $this.$x.a - T::one()).div_euclid($this.$x.a);
                $this.k_range.0 = $this.k_range.0.max(t);
            } else {
                // k <= (l - b) / a
                let t = ($this.$x.b - l).div_euclid(-$this.$x.a);
                $this.k_range.1 = $this.k_range.1.min(t);
            }
        }
        if !r.is_maximum() {
            if $this.$x.a.is_positive() {
                // k <= (r - b) / a
                let t = (r - $this.$x.b).div_euclid($this.$x.a);
                $this.k_range.1 = $this.k_range.1.min(t);
            } else {
                // (r - b) / a <= k
                let t = ($this.$x.b - r - $this.$x.a - T::one()).div_euclid(-$this.$x.a);
                $this.k_range.0 = $this.k_range.0.max(t);
            }
        }
    };
}

impl<T> LinearDiophantineSolution<T>
where
    T: Signed,
{
    pub fn eval(&self, k: T) -> (T, T) {
        (self.x.eval(k), self.y.eval(k))
    }
    pub fn with_x_range<R>(mut self, range: R) -> Self
    where
        R: RangeBoundsExt<T>,
    {
        with_range!(self, x, range);
        self
    }
    pub fn with_y_range<R>(mut self, range: R) -> Self
    where
        R: RangeBoundsExt<T>,
    {
        with_range!(self, y, range);
        self
    }
    pub fn with_x_order(mut self) -> Self {
        if self.x.a.is_negative() {
            self.x.a = -self.x.a;
            self.y.a = -self.y.a;
            self.k_range = (
                if self.k_range.1 == T::maximum() {
                    T::minimum()
                } else {
                    -self.k_range.1
                },
                if self.k_range.0 == T::minimum() {
                    T::maximum()
                } else {
                    -self.k_range.0
                },
            );
        }
        self
    }
    pub fn with_y_order(mut self) -> Self {
        if self.y.a.is_negative() {
            self.x.a = -self.x.a;
            self.y.a = -self.y.a;
            self.k_range = (
                if self.k_range.1 == T::maximum() {
                    T::minimum()
                } else {
                    -self.k_range.1
                },
                if self.k_range.0 == T::minimum() {
                    T::maximum()
                } else {
                    -self.k_range.0
                },
            );
        }
        self
    }
    pub fn k_range(&self) -> RangeInclusive<T> {
        self.k_range.0..=self.k_range.1
    }
}

/// Solve ax + by = c
pub fn solve_linear_diophantine<T>(a: T, b: T, c: T) -> Option<LinearDiophantineSolution<T>>
where
    T: Signed,
{
    assert!(!a.is_zero(), "a must be non-zero");
    assert!(!b.is_zero(), "b must be non-zero");
    let ExtendedGcd { g, x: x0, y: y0 } = a.extgcd(b);
    let g = g.signed();
    let a = a / g;
    let b = b / g;
    if c.is_zero() {
        return Some(LinearDiophantineSolution {
            x: Linear::new(b, T::zero()),
            y: Linear::new(-a, T::zero()),
            k_range: (T::minimum(), T::maximum()),
        });
    }
    if !(c % g).is_zero() {
        return None;
    }
    let c = c / g;
    let x = Linear::new(b, x0 * c);
    let y = Linear::new(-a, y0 * c);
    Some(LinearDiophantineSolution {
        x,
        y,
        k_range: (T::minimum(), T::maximum()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{num::Zero, rand, tools::Xorshift};

    #[test]
    fn test_solve_linear_diophantine() {
        let mut rng = Xorshift::default();
        for t in [2, 10, 100, 1_000_000i64] {
            rand!(rng, abc: [(-t..=t, -t..=t, -t..=t); 100]);
            for (a, b, c) in abc {
                if a.is_zero() || b.is_zero() {
                    continue;
                }
                if let Some(sol) = solve_linear_diophantine(a, b, c) {
                    for k in -10..=10 {
                        let (x, y) = sol.eval(k);
                        assert_eq!(a * x + b * y, c);
                    }
                    rand!(rng, lr: [(-100i64..=100, -100i64..=100); 100]);
                    for (l, r) in lr {
                        let mut sol = sol;
                        sol = sol.with_x_range(l..=r);
                        let sorted = rng.gen_bool(0.5);
                        if sorted {
                            sol = sol.with_x_order();
                        }
                        for k in sol.k_range().clone() {
                            let (x, y) = sol.eval(k);
                            assert_eq!(a * x + b * y, c);
                            assert!((l..=r).contains(&x));
                        }
                        for k in -100..=100 {
                            let (x, y) = sol.eval(k);
                            assert_eq!(a * x + b * y, c);
                            assert_eq!((l..=r).contains(&x), sol.k_range().contains(&k));
                        }
                        if sorted {
                            assert!(sol.k_range().map(|k| sol.eval(k).0).is_sorted());
                        }
                    }
                    rand!(rng, lr: [(-100i64..=100, -100i64..=100); 100]);
                    for (l, r) in lr {
                        let mut sol = sol;
                        sol = sol.with_y_range(l..=r);
                        let sorted = rng.gen_bool(0.5);
                        if sorted {
                            sol = sol.with_y_order();
                        }
                        for k in sol.k_range().clone() {
                            let (x, y) = sol.eval(k);
                            assert_eq!(a * x + b * y, c);
                            assert!((l..=r).contains(&y));
                        }
                        for k in -100..=100 {
                            let (x, y) = sol.eval(k);
                            assert_eq!(a * x + b * y, c);
                            assert_eq!((l..=r).contains(&y), sol.k_range().contains(&k));
                        }
                        if sorted {
                            assert!(sol.k_range().map(|k| sol.eval(k).1).is_sorted());
                        }
                    }
                } else {
                    let ExtendedGcd { g, .. } = a.extgcd(b);
                    assert!(!(c % g.signed()).is_zero());
                    for x in -100..=100 {
                        let y = (c - a * x).div_euclid(b);
                        assert_ne!(a * x + b * y, c);
                    }
                }
            }
        }
    }
}
