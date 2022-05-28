#[derive(Debug, Default, Clone, Copy)]
pub struct PartialIgnoredOrd<T, U>(pub T, pub U);
impl<T: Eq, U> Eq for PartialIgnoredOrd<T, U> {}
impl<T, U> PartialEq for PartialIgnoredOrd<T, U>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
impl<T, U> PartialOrd for PartialIgnoredOrd<T, U>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
impl<T, U> Ord for PartialIgnoredOrd<T, U>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
