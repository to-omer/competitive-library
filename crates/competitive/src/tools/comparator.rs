use std::cmp::Ordering;

pub trait Comparator<T> {
    fn compare(&mut self, a: &T, b: &T) -> Ordering;
}

impl<T, F> Comparator<T> for F
where
    F: FnMut(&T, &T) -> Ordering,
{
    fn compare(&mut self, a: &T, b: &T) -> Ordering {
        (self)(a, b)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Less;
impl<T> Comparator<T> for Less
where
    T: Ord,
{
    fn compare(&mut self, a: &T, b: &T) -> Ordering {
        a.cmp(b)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Greater;
impl<T> Comparator<T> for Greater
where
    T: Ord,
{
    fn compare(&mut self, a: &T, b: &T) -> Ordering {
        b.cmp(a)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByKey<F>(pub F);
impl<T, F, K> Comparator<T> for ByKey<F>
where
    F: FnMut(&T) -> K,
    K: Ord,
{
    fn compare(&mut self, a: &T, b: &T) -> Ordering {
        (self.0)(a).cmp(&(self.0)(b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    #[test]
    fn test_comparators() {
        let mut rng = Xorshift::default();
        for _ in 0..10_000 {
            let a = rng.random(-100i32..=100);
            let b = rng.random(-100i32..=100);
            let mut cmp = |a: &i32, b: &i32| b.cmp(a);
            assert_eq!(cmp.compare(&a, &b), b.cmp(&a));
            assert_eq!(Less.compare(&a, &b), a.cmp(&b));
            assert_eq!(Greater.compare(&a, &b), b.cmp(&a));
            let divisor = rng.random(1..=100);
            assert_eq!(
                ByKey(|x: &i32| x.rem_euclid(divisor)).compare(&a, &b),
                a.rem_euclid(divisor).cmp(&b.rem_euclid(divisor))
            );
        }
    }
}
