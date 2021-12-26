#[codesnip::entry("GetDistinctMut")]
pub trait GetDistinctMut<I> {
    type Output;
    fn get_distinct_mut(self, index: I) -> Self::Output;
}
#[codesnip::entry("GetDistinctMut")]
impl<'a, T> GetDistinctMut<(usize, usize)> for &'a mut [T] {
    type Output = (&'a mut T, &'a mut T);
    fn get_distinct_mut(self, (i0, i1): (usize, usize)) -> Self::Output {
        assert_ne!(i0, i1);
        assert!(i0 < self.len());
        assert!(i1 < self.len());
        let ptr = self.as_mut_ptr();
        unsafe { (&mut *ptr.add(i0), &mut *ptr.add(i1)) }
    }
}
#[codesnip::entry("GetDistinctMut")]
impl<'a, T> GetDistinctMut<(usize, usize, usize)> for &'a mut [T] {
    type Output = (&'a mut T, &'a mut T, &'a mut T);
    fn get_distinct_mut(self, (i0, i1, i2): (usize, usize, usize)) -> Self::Output {
        assert_ne!(i0, i1);
        assert_ne!(i0, i2);
        assert!(i0 < self.len());
        assert!(i1 < self.len());
        assert!(i2 < self.len());
        let ptr = self.as_mut_ptr();
        unsafe { (&mut *ptr.add(i0), &mut *ptr.add(i1), &mut *ptr.add(i2)) }
    }
}
