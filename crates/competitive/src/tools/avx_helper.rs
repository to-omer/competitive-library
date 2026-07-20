use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SimdBackend {
    Scalar,
    Avx2,
    Avx512,
}

static AVX512_ENABLED: AtomicBool = AtomicBool::new(true);

#[inline]
pub fn disable_avx512() {
    AVX512_ENABLED.store(false, Ordering::Relaxed);
}

#[inline]
pub fn enable_avx512() {
    AVX512_ENABLED.store(true, Ordering::Relaxed);
}

#[inline]
pub fn avx512_enabled() -> bool {
    AVX512_ENABLED.load(Ordering::Relaxed)
}

#[inline]
pub fn avx512_supported() -> bool {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    return is_x86_feature_detected!("avx512f")
        && is_x86_feature_detected!("avx512dq")
        && is_x86_feature_detected!("avx512cd")
        && is_x86_feature_detected!("avx512bw")
        && is_x86_feature_detected!("avx512vl");
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    false
}

#[inline]
pub fn simd_backend() -> SimdBackend {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if avx512_enabled() && avx512_supported() {
            return SimdBackend::Avx512;
        }
        if is_x86_feature_detected!("avx2") {
            return SimdBackend::Avx2;
        }
    }
    SimdBackend::Scalar
}

#[macro_export]
macro_rules! avx_helper {
    (@dispatch $backend:path, $kind:ident; $avx512:expr, $avx2:expr, $scalar:expr) => {{
        #[cfg(target_arch = "x86_64")]
        {
            match $backend() {
                $kind::Avx512 => $avx512,
                $kind::Avx2 => $avx2,
                $kind::Scalar => $scalar,
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        $scalar
    }};
    (@dispatch_avx2_fma $avx2:expr, $scalar:expr) => {{
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
                $avx2
            } else {
                $scalar
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        $scalar
    }};
    (@avx512 $(#[$meta:meta])* $vis:vis fn $name:ident$(<$($T:ident),+>)?($($i:ident: $t:ty),*) -> $ret:ty where [$($clauses:tt)*] $body:block) => {
        $(#[$meta])*
        $vis fn $name$(<$($T)*>)?($($i: $t),*) -> $ret
        where
            $($clauses)*
        {
            if $crate::avx512_supported() {
                $crate::avx_helper!(@def_avx512 fn avx512$(<$($T)*>)?($($i: $t),*) -> $ret where [$($clauses)*] $body);
                unsafe { avx512$(::<$($T),*>)?($($i),*) }
            } else if is_x86_feature_detected!("avx2") {
                $crate::avx_helper!(@def_avx2 fn avx2$(<$($T)*>)?($($i: $t),*) -> $ret where [$($clauses)*] $body);
                unsafe { avx2$(::<$($T),*>)?($($i),*) }
            } else {
                $body
            }
        }
    };
    (@avx2 $(#[$meta:meta])* $vis:vis fn $name:ident$(<$($T:ident),+>)?($($i:ident: $t:ty),*) -> $ret:ty where [$($clauses:tt)*] $body:block) => {
        $(#[$meta])*
        $vis fn $name$(<$($T)*>)?($($i: $t),*) -> $ret
        where
            $($clauses)*
        {
            if is_x86_feature_detected!("avx2") {
                $crate::avx_helper!(@def_avx2 fn avx2$(<$($T)*>)?($($i: $t),*) -> $ret where [$($clauses)*] $body);
                unsafe { avx2$(::<$($T),*>)?($($i),*) }
            } else {
                $body
            }
        }
    };
    (@def_avx512 fn $name:ident$(<$($T:ident),+>)?($($args:tt)*) -> $ret:ty where [$($clauses:tt)*] $body:block) => {
        #[target_feature(enable = "avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]
        unsafe fn $name$(<$($T)*>)?($($args)*) -> $ret
        where
            $($clauses)*
        $body
    };
    (@def_avx2 fn $name:ident$(<$($T:ident),+>)?($($args:tt)*) -> $ret:ty where [$($clauses:tt)*] $body:block) => {
        #[target_feature(enable = "avx2")]
        unsafe fn $name$(<$($T)*>)?($($args)*) -> $ret
        where
            $($clauses)*
        $body
    };
    (@$tag:ident $(#[$meta:meta])* $vis:vis fn $name:ident$(<$($T:ident),+>)?($($args:tt)*) -> $ret:ty $body:block) => {
        $crate::avx_helper!(@$tag $(#[$meta])* $vis fn $name$(<$($T)*>)?($($args)*) -> $ret where [] $body);
    };
    (@$tag:ident $(#[$meta:meta])* $vis:vis fn $name:ident$(<$($T:ident),+>)?($($args:tt)*) $($t:tt)*) => {
        $crate::avx_helper!(@$tag $(#[$meta])* $vis fn $name$(<$($T)*>)?($($args)*) -> () $($t)*);
    };
    ($($t:tt)*) => {
        ::std::compile_error!($($t)*);
    }
}
