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

    #[test]
    fn test_fn_comparator() {
        let mut cmp = |a: &i32, b: &i32| b.cmp(a);
        assert_eq!(cmp.compare(&1, &2), Ordering::Greater);
        assert_eq!(cmp.compare(&2, &1), Ordering::Less);
        assert_eq!(cmp.compare(&1, &1), Ordering::Equal);
    }

    #[test]
    fn test_less_comparator() {
        let mut cmp = Less;
        assert_eq!(cmp.compare(&1, &2), Ordering::Less);
        assert_eq!(cmp.compare(&2, &1), Ordering::Greater);
        assert_eq!(cmp.compare(&1, &1), Ordering::Equal);
    }

    #[test]
    fn test_greater_comparator() {
        let mut cmp = Greater;
        assert_eq!(cmp.compare(&1, &2), Ordering::Greater);
        assert_eq!(cmp.compare(&2, &1), Ordering::Less);
        assert_eq!(cmp.compare(&1, &1), Ordering::Equal);
    }

    #[test]
    fn test_by_key_comparator() {
        let mut cmp = ByKey(|x: &i32| -x);
        assert_eq!(cmp.compare(&1, &2), Ordering::Greater);
        assert_eq!(cmp.compare(&2, &1), Ordering::Less);
        assert_eq!(cmp.compare(&1, &1), Ordering::Equal);
    }
}
