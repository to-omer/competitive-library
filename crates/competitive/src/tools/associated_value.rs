/// Trait for a modifiable value associated with a type.
pub trait AssociatedValue {
    /// Type of value.
    type T: 'static + Clone;
    fn local_key() -> &'static std::thread::LocalKey<std::cell::UnsafeCell<Self::T>>;
    #[inline]
    fn get() -> Self::T {
        Self::with(Clone::clone)
    }
    #[inline]
    fn set(x: Self::T) {
        Self::local_key().with(|cell| unsafe { *cell.get() = x })
    }
    #[inline]
    fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&Self::T) -> R,
    {
        Self::local_key().with(|cell| unsafe { f(&*cell.get()) })
    }
    #[inline]
    fn modify<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self::T) -> R,
    {
        Self::local_key().with(|cell| unsafe { f(&mut *cell.get()) })
    }
}

mod associated_value_macros {
    /// Implement [`AssociatedValue`].
    ///
    /// [`AssociatedValue`]: super::AssociatedValue
    ///
    /// # Examples
    ///
    /// ```
    /// use competitive::tools::AssociatedValue;
    /// struct X;
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
    /// struct X;
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
                fn local_key() -> &'static ::std::thread::LocalKey<::std::cell::UnsafeCell<Self::T>> {
                    ::std::thread_local!(static __LOCAL_KEY: ::std::cell::UnsafeCell<$t> = ::std::cell::UnsafeCell::new($e));
                    &__LOCAL_KEY
                }
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impl_assoc_value;

    #[test]
    fn test_associated_value() {
        struct X;
        impl_assoc_value!(X, usize);
        X::set(10);
        assert_eq!(X::get(), 10);
        assert_eq!(X::with(|x| x + 1), 11);
        X::modify(|x| *x += 1);
        assert_eq!(X::get(), 11);
    }
}
