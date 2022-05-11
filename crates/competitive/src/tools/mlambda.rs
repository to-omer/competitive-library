/// Macro that define closure like macro. Unlike closure, this macro localizes variable capture.
///
/// # Example
/// ```
/// # use competitive::mlambda;
/// let graph: Vec<Vec<usize>> = vec![vec![1, 2], vec![2], vec![]];
/// let mut deq = std::collections::VecDeque::new();
/// let mut dist: Vec<usize> = vec![!0; 3];
/// mlambda!(
///     fn push(v: usize, cost: usize) {
///         if dist[v] > cost {
///             dist[v] = cost;
///             deq.push_back(v);
///         }
///     }
/// );
/// push!(0, 0);
/// while let Some(v) = deq.pop_front() {
///     for &to in &graph[v] {
///         push!(to, dist[v] + 1);
///     }
/// }
/// assert_eq!(vec![0, 1, 1], dist);
/// ```
#[macro_export]
macro_rules! mlambda {
    (
        @def ($dol:tt) [$([$x:ident])*][$([$y:ident, $($z:tt)*])*]
        fn $name:ident($($args:tt)*) -> $ret:ty $body:block
    ) => {
        macro_rules! $name {
            ($($dol $x:expr),* $dol(,)?) => {{
                $(let $y $($z)* = $dol $y;)*
                $body
            }}
        }
    };
    (@pre () [$($x:tt)*][$($y:tt)*] fn $name:ident($($args:tt)*) -> $ret:ty $body:block) => {
        $crate::mlambda!(@def ($) [$($x)*][$($y)*] fn $name($($args)*) -> $ret $body)
    };
    (@pre () [$($x:tt)*][$($y:tt)*] fn $name:ident($($args:tt)*) $body:block) => {
        $crate::mlambda!(@pre () [$($x)*][$($y)*] fn $name($($args)*) -> () $body)
    };
    (@pre ($arg:ident $(:$ty:ty)?) [$($x:tt)*][$($y:tt)*] $($rest:tt)*) => {
        $crate::mlambda!(@pre () [$($x)* [$arg]][$($y)* [$arg, $(:$ty)?]] $($rest)*)
    };
    (@pre ($arg:ident $(:$ty:ty)?, $($args:tt)*) [$($x:tt)*][$($y:tt)*] $($rest:tt)*) => {
        $crate::mlambda!(@pre ($($args)*) [$($x)* [$arg]][$($y)* [$arg, $(:$ty)?]] $($rest)*)
    };
    (fn $name:ident($($args:tt)*) $($rest:tt)*) => {
        $crate::mlambda!(@pre ($($args)*) [][] fn $name($($args)*) $($rest)*)
    };
}
