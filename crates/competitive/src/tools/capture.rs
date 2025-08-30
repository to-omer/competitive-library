/// Macro that returns a recursive function that (semi-)automatically captures.
///
/// # Example
/// default version
/// ```
/// # use competitive::crecurse;
/// let mut res = 0usize;
/// let coeff = 3usize;
/// crecurse!(
///     // (1) semi-automatically capture mutable reference (res: &mut usize)
///     [res: usize],
///     fn mul(x: usize, y: usize) {
///         if y > 0 {
///             if y % 2 == 1 {
///                 // (2) automatically capture reference (coeff: &usize)
///                 *res += coeff * x;
///             }
///             // (3) call macro to recurse
///             mul!(x + x, y / 2);
///         }
///     }
/// )(10, 19); // (4) macro returns captured version of the recursive function
/// assert_eq!(res, coeff * 10 * 19);
/// ```
///
/// unsafe version (automatically capture everything)
/// ```
/// # use competitive::crecurse;
/// let mut res = 0usize;
/// let coeff = 3usize;
/// crecurse!(
///     unsafe fn mul(x: usize, y: usize) {
///         if y > 0 {
///             if y % 2 == 1 {
///                 res += coeff * x;
///             }
///             mul!(x + x, y / 2);
///         }
///     }
/// )(10, 19);
/// assert_eq!(res, coeff * 10 * 19);
/// ```
///
/// no overhead version (semi-automatically capture everything)
/// ```
/// # use competitive::crecurse;
/// let mut res = 0usize;
/// let coeff = 3usize;
/// crecurse!(
///     [res: &mut usize, coeff: &usize],
///     static fn mul(x: usize, y: usize) {
///         if y > 0 {
///             if y % 2 == 1 {
///                 *res += coeff * x;
///             }
///             mul!(x + x, y / 2);
///         }
///     }
/// )(10, 19);
/// assert_eq!(res, coeff * 10 * 19);
/// ```
///
/// # Syntax
/// ```txt
/// crecurse!(
///     ([($ident: $type),*,?],)?
///     (unsafe|static)? fn $ident\(($ident: $type),*,?\) (-> $type)? $block
/// )
/// ```
#[macro_export]
macro_rules! crecurse {
    (@macro_def ($dol:tt) $name:ident $($cargs:ident)*) => {
        #[allow(unused_macros)]
        macro_rules! $name { ($dol($dol args:expr),*) => { $name($dol($dol args,)* $($cargs,)* ) } }
    };

    (
        @static [$(($cargs:ident, $cargsexpr:expr, $cargsty:ty))*] [$(,)?],
        fn $func:ident ($($args:ident: $argsty:ty),* $(,)?) -> $ret:ty $body:block
    ) => {{
        fn $func($($args: $argsty,)* $($cargs: $cargsty,)*) -> $ret {
            $crate::crecurse!(@macro_def ($) $func $($cargs)*);
            $body
        }
        |$($args: $argsty,)*| -> $ret { $func($($args,)* $($cargsexpr,)*) }
    }};
    (@static [$($pcaps:tt)*] [$(,)?], fn $func:ident ($($argstt:tt)*) $($rest:tt)*) => {
        $crate::crecurse!(@static [$($pcaps)*] [], fn $func ($($argstt)*) -> () $($rest)*)
    };
    (@static [$($pcaps:tt)*] [$carg:ident: &mut $cargty:ty, $($caps:tt)*], $($rest:tt)*) => {
        $crate::crecurse!(@static [$($pcaps)* ($carg, &mut $carg, &mut $cargty)] [$($caps)*], $($rest)*)
    };
    (@static [$($pcaps:tt)*] [$carg:ident: &$cargty:ty, $($caps:tt)*], $($rest:tt)*) => {
        $crate::crecurse!(@static [$($pcaps)* ($carg, &$carg, &$cargty)] [$($caps)*], $($rest)*)
    };
    (@static [$($pcaps:tt)*] [$carg:ident: $cargty:ty, $($caps:tt)*], $($rest:tt)*) => {
        $crate::crecurse!(@static [$($pcaps)* ($carg, $carg, $cargty)] [$($caps)*], $($rest)*)
    };
    ($([$($caps:tt)*],)? static fn $func:ident ($($args:ident: $argsty:ty),* $(,)?) $($rest:tt)*) => {
        $crate::crecurse!(@static [] [$($($caps)*)?,], fn $func ($($args: $argsty),*) $($rest)*)
    };

    (
        @default [$($cargs:ident: $cargsty:ty),* $(,)?],
        fn $func:ident ($($args:ident: $argsty:ty),* $(,)?) -> $ret:ty $body:block
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
                    $crate::crecurse!(@macro_def ($) $func $($cargs)*);
                    $body
                },
                $($args,)* $(&mut $cargs,)*
            )
        }
    }};
    (@default [$($caps:tt)*], fn $func:ident ($($argstt:tt)*) $($rest:tt)*) => {
        $crate::crecurse!(@default [$($caps)*], fn $func ($($argstt)*) -> () $($rest)*)
    };
    ($([$($caps:tt)*],)? fn $func:ident ($($args:ident: $argsty:ty),* $(,)?) $($rest:tt)*) => {
        $crate::crecurse!(@default [$($($caps)*)?], fn $func ($($args: $argsty),*) $($rest)*)
    };

    (
        @unsafe [$($cargs:ident: $cargsty:ty),* $(,)?],
        fn $func:ident ($($args:ident: $argsty:ty),* $(,)?) -> $ret:ty $body:block
    ) => {{
        fn call<F>(f: &mut F, $($args: $argsty,)* $($cargs: &mut $cargsty,)*) -> $ret
        where
            F: FnMut(&mut dyn FnMut($($argsty,)* $(&mut $cargsty,)*) -> $ret, $($argsty,)* $(&mut $cargsty,)*) -> $ret,
        {
            let fp = f as *mut F;
            (unsafe { &mut *fp })(
                &mut |$($args: $argsty,)* $($cargs: &mut $cargsty,)*| -> $ret {
                    call(unsafe { &mut *fp }, $($args,)* $($cargs,)*)
                },
                $($args,)* $($cargs,)*
            )
        }
        |$($args: $argsty,)*| -> $ret {
            call(
                &mut |$func, $($args: $argsty,)* $($cargs: &mut $cargsty,)*| -> $ret {
                    $crate::crecurse!(@macro_def ($) $func $($cargs)*);
                    $body
                },
                $($args,)* $(&mut $cargs,)*
            )
        }
    }};

    (@unsafe [$($caps:tt)*], fn $func:ident ($($argstt:tt)*) $($rest:tt)*) => {
        $crate::crecurse!(@unsafe [$($caps)*], fn $func ($($argstt)*) -> () $($rest)*)
    };
    ($([$($caps:tt)*],)? unsafe fn $func:ident ($($args:ident: $argsty:ty),* $(,)?) $($rest:tt)*) => {
        $crate::crecurse!(@unsafe [$($($caps)*)?], fn $func ($($args: $argsty),*) $($rest)*)
    };
    ($($t:tt)*) => {
        ::std::compile_error!(::std::concat!("invalid input: ", ::std::stringify!($($t)*)))
    };
}

/// Automatic memorization for recursive functions.
///
/// This macro binds memorized version of the recursive functions to a local variable.
/// The specification of the function declaration part is the same as [`crecurse`].
///
/// [`crecurse`]: crate::crecurse
///
/// # Example
/// ```
/// # use competitive::memorize;
/// memorize!(
///     fn comb(n: usize, r: usize) -> usize {
///         if r > n {
///             0
///         } else if r == 0 || r == n {
///             1
///         } else {
///             comb!(n - 1, r) + comb!(n - 1, r - 1)
///         }
///     }
/// );
/// assert_eq!(comb(30, 12), 86493225);
/// ```
#[macro_export]
macro_rules! memorize {
    (
        @inner [$map:ident, $Map:ty, $init:expr]
        fn $name:ident ($($args:ident: $argsty:ty),* $(,)?) -> $ret:ty $body:block
    ) => {
        let mut $map: $Map = $init;
        #[allow(unused_mut)]
        let mut $name = $crate::crecurse!(
            [$map: $Map],
            fn $name ($($args: $argsty),*) -> $ret {
                if let Some(value) = $map.get(&($(Clone::clone(&$args),)*)).cloned() {
                    value
                } else {
                    let value = (|| $body)();
                    $map.insert(($($args,)*), value.clone());
                    value
                }
            }
        );
    };
    (fn $name:ident ($($args:ident: $argsty:ty),* $(,)?) -> $ret:ty $body:block) => {
        $crate::memorize!(
            @inner [
                __memorize_map,
                ::std::collections::HashMap<($($argsty,)*), $ret>,
                ::std::default::Default::default()
            ]
            fn $name ($($args: $argsty),*) -> $ret $body
        );
    }
}
