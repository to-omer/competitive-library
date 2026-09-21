/// Trait for a modifiable value associated with a type.
pub trait AssociatedValue {
    /// Type of value.
    type T: 'static + Clone;
    unsafe fn __local_key() -> &'static std::thread::LocalKey<std::cell::Cell<Self::T>>;
    #[inline]
    fn get() -> Self::T {
        Self::with(Clone::clone)
    }
    #[inline]
    fn set(x: Self::T) {
        unsafe { Self::__local_key().with(|cell| cell.set(x)) }
    }
    #[inline]
    fn replace(x: Self::T) -> Self::T {
        unsafe { Self::__local_key().with(|cell| cell.replace(x)) }
    }
    #[inline]
    fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&Self::T) -> R,
    {
        unsafe { Self::__local_key().with(|cell| f(&*cell.as_ptr())) }
    }
    #[inline]
    fn modify<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self::T) -> R,
    {
        unsafe { Self::__local_key().with(|cell| f(&mut *cell.as_ptr())) }
    }
}

/// Implement [`AssociatedValue`].
///
/// [`AssociatedValue`]: super::AssociatedValue
///
/// # Examples
///
/// ```
/// use competitive::tools::AssociatedValue;
/// enum X {}
/// competitive::impl_assoc_value!(X, usize, 1);
/// assert_eq!(X::get(), 1);
/// X::set(10);
/// assert_eq!(X::get(), 10);
/// ```
///
/// init with `Default::default()`
///
/// ```
/// use competitive::tools::AssociatedValue;
/// enum X {}
/// competitive::impl_assoc_value!(X, usize);
/// assert_eq!(X::get(), Default::default());
/// ```
#[macro_export]
macro_rules! impl_assoc_value {
    ($name:ident, $t:ty) => {
        $crate::impl_assoc_value!($name, $t, Default::default());
    };
    ($name:ident, $t:ty, $e:expr) => {
        impl AssociatedValue for $name {
            type T = $t;
            #[inline]
            unsafe fn __local_key() -> &'static ::std::thread::LocalKey<::std::cell::Cell<Self::T>> {
                ::std::thread_local!(static __LOCAL_KEY: ::std::cell::Cell<$t> = ::std::cell::Cell::new($e));
                &__LOCAL_KEY
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    #[test]
    fn test_associated_value() {
        enum X {}
        impl_assoc_value!(X, usize);
        let mut rng = Xorshift::default();
        let mut expected = 0usize;
        for _ in 0..1000 {
            let value = rng.random(..);
            match rng.random(0..3) {
                0 => {
                    X::set(value);
                    expected = value;
                }
                1 => {
                    assert_eq!(X::replace(value), expected);
                    expected = value;
                }
                _ => {
                    X::modify(|x| *x ^= value);
                    expected ^= value;
                }
            }
            assert_eq!(X::get(), expected);
            assert_eq!(X::with(|x| x.count_ones()), expected.count_ones());
        }
    }
}
