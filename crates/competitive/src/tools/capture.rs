#[macro_export]
macro_rules! capture {
    ([$($ca:tt)*], fn $name:ident($($arg:tt)*) -> $ret:ty $body:block) => {
        capture!({}[$($ca)*,] fn $name($($arg)*) -> $ret $body)
    };
    ([$($ca:tt)*], fn $name:ident($($arg:tt)*) $body:block) => {
        capture!({}[$($ca)*,] fn $name($($arg)*) -> () $body)
    };
    ({$(($g:ident, $ga:expr, $gt:ty))*}[] fn $name:ident($($a:ident: $at:ty),*) -> $ret:ty $body:block) => {
        fn $name($($g: $gt,)* $($a: $at,)*) -> $ret {
            #[allow(unused_macros)]
            macro_rules! $name {
                () => {
                    |$($a),*| $name($($g,)* $($a,)*)
                }
            }
            $body
        }
        #[allow(unused_mut)]
        let mut $name = |$($a),*| $name($($ga,)* $($a,)*);
    };
    ({$($g:tt)*}[]fn $name:ident($($a:ident: $at:ty),*,) $($rest:tt)*) => {
        capture!({$($g)*}[]fn $name($($a: $at),*) $($rest)*)
    };
    ({$($done:tt)*}[,] $($rest:tt)*) => {
        capture!({$($done)*}[] $($rest)*)
    };
    ({$($done:tt)*}[$g:ident: &mut $gt:ty, $($rest:tt)*] $($info:tt)*) => {
        capture!({$($done)* ($g, &mut $g, &mut $gt)}[$($rest)*] $($info)*)
    };
    ({$($done:tt)*}[$g:ident: &$gt:ty, $($rest:tt)*] $($info:tt)*) => {
        capture!({$($done)* ($g, &$g, &$gt)}[$($rest)*] $($info)*)
    };
    ({$($done:tt)*}[$g:ident: $gt:ty, $($rest:tt)*] $($info:tt)*) => {
        capture!({$($done)* ($g, $g, $gt)}[$($rest)*]$($info)*)
    };
    ({$($done:tt)*}[$g:ident, $($rest:tt)*] $($info:tt)*) => {
        capture!({$($done)* ($g, $g, usize)}[$($rest)*]$($info)*)
    };
}

#[macro_export]
macro_rules! crecurse {
    (
        [$($cargs:ident: $cargsty:ty),*],
        fn $func:ident ($($args:ident: $argsty:ty),*) -> $ret:ty $body:block
    ) => {{
        fn call<F>(f: &F, $($args: $argsty,)* $($cargs: &mut $cargsty,)*) -> $ret
        where
            F: Fn(&dyn Fn($($argsty,)* $(&mut $cargsty,)*) -> $ret, $($argsty,)* $(&mut $cargsty,)*) -> $ret,
        {
            f(
                &|$($args: $argsty,)* $($cargs: &mut $cargsty,)*| -> $ret {
                    call(f, $($args,)* $($cargs,)*)
                },
                $($args,)* $($cargs,)*
            )
        }
        |$($args: $argsty,)*| -> $ret {
            call(
                &|$func, $($args: $argsty,)* $($cargs: &mut $cargsty,)*| -> $ret {
                    #[allow(unused_macros)]
                    macro_rules! $func {
                        () => {
                            |$($args: $argsty,)*| -> $ret {
                                $func($($args,)* $($cargs,)*)
                            }
                        }
                    }
                    $body
                },
                $($args,)* $(&mut $cargs,)*
            )
        }
    }};
    (fn $($rest:tt)*) => {
        crecurse!([], fn $($rest)*)
    };
    ([$($caps:tt)*], fn $func:ident ($($args:ident: $argsty:ty),*) $body:block) => {
        crecurse!([$($caps)*], fn $func($($args: $argsty),*) -> () $body)
    };
    ([$($caps:tt)*], fn $func:ident ($($args:ident: $argsty:ty),*,) $($rest:tt)*) => {
        crecurse!([$($caps)*], fn $func($($args: $argsty),*) $($rest)*)
    };
}
