#[macro_export]
macro_rules! invariant {
    ($e:expr) => {
        debug_assert!($e);
        if !$e {
            unsafe { ::core::hint::unreachable_unchecked() }
        }
    };
}
