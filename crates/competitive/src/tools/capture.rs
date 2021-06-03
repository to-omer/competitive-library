/// Macro that returns a recursive function that automatically captures references
/// and semi-automatically captures mutable references.
///
/// # Example
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
///             // (3) call macro to get captured version of the recursive function
///             // internally, `mul!()` returns `|x, y| mul(x, y, res)`
///             mul!()(x + x, y / 2);
///         }
///     }
/// )(10, 19); // (4) macro returns captured version of the recursive function
/// assert_eq!(res, coeff * 10 * 19);
/// ```
///
/// # Syntax
/// ```txt
/// crecurse!(
///     ([($ident: $type),*,?],)?
///     fn $ident\(($ident: $type),*\) (-> $type)? $block
/// )
/// ```
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
        $crate::crecurse!([], fn $($rest)*)
    };
    ([$($caps:tt)*], fn $func:ident ($($args:ident: $argsty:ty),*) $body:block) => {
        $crate::crecurse!([$($caps)*], fn $func($($args: $argsty),*) -> () $body)
    };
    ([$($caps:tt)*], fn $func:ident ($($args:ident: $argsty:ty),*,) $($rest:tt)*) => {
        $crate::crecurse!([$($caps)*], fn $func($($args: $argsty),*) $($rest)*)
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
///             comb!()(n - 1, r) + comb!()(n - 1, r - 1)
///         }
///     }
/// );
/// assert_eq!(comb(10, 4), 210);
/// ```
#[macro_export]
macro_rules! memorize {
    (fn $name:ident ($($args:ident: $argsty:ty),*) -> $ret:ty $body:block) => {
        let mut __memorize_cache = ::std::collections::HashMap::<($($argsty),*), $ret>::new();
        #[allow(unused_mut)]
        let mut $name = $crate::crecurse!(
            [__memorize_cache: ::std::collections::HashMap::<($($argsty),*), $ret>],
            fn $name ($($args: $argsty),*) -> $ret {
                if let Some(__value) = __memorize_cache.get(&($($args),*)).cloned() {
                    __value
                } else {
                    let __value = $body;
                    __memorize_cache.insert(($($args),*), __value.clone());
                    __value
                }
            }
        );
    };
}
