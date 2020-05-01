use cargo_snippet::snippet;

#[snippet("comprehension")]
#[allow(unused_macros)]
macro_rules! comprehension {
    ($it:expr; @$type:ty) => {
        $it.collect::<$type>()
    };
    ($it:expr) => {
        comprehension![$it; @Vec<_>;]
    };
    ($it:expr; @$type:ty; $p:pat => $e:expr) => {
        comprehension![$it.map(|$p| $e); $type]
    };
    ($it:expr; $p:pat => $e:expr) => {
        comprehension![$it; @Vec<_>; $p => $e]
    };
    ($it:expr; $p:pat => $e:expr, $b:expr) => {
        comprehension![$it.filter_map(|$p| if $b { Some($e) } else { None }); @Vec<_>]
    };
    ($it:expr; $p:pat => $e:expr, $b1:expr, $b2:expr) => {
        comprehension![$it; $p => $e, $b1 & $b2];
    };
    ($it:expr; $p:pat => $e:expr, $b1:expr, $b2:expr, $($t:tt)*) => {
        comprehension![$it; $p => $e, $b1 & $b2, $($t)*]
    };
}
