#[macro_export]
macro_rules! comprehension {
    ($it:expr; @$type:ty) => {
        $it.collect::<$type>()
    };
    ($it:expr) => {
        comprehension![$it; @Vec<_>]
    };
    ($it:expr; @$type:ty; $p:pat => $e:expr) => {
        comprehension![$it.map(|$p| $e); @$type]
    };
    ($it:expr; $p:pat => $($t:tt)*) => {
        comprehension![$it; @Vec<_>; $p => $($t)*]
    };
    ($it:expr; $p:pat, $($t:tt)*) => {
        comprehension![$it; @Vec<_>; $p, $($t)*]
    };
    ($it:expr; @$type:ty; $p:pat => $e:expr) => {
        comprehension![$it; @$type; $p => $e]
    };
    ($it:expr; @$type:ty; $p:pat, $b:expr) => {
        comprehension![$it.filter(|$p| $b); @$type]
    };
    ($it:expr; @$type:ty; $p:pat => $e:expr, $b:expr) => {
        comprehension![$it.filter_map(|$p| if $b { Some($e) } else { None }); @$type]
    };
    ($it:expr; @$type:ty; $p:pat => $e:expr, $b1:expr, $b2:expr) => {
        comprehension![$it; @$type; $p => $e, $b1 & $b2]
    };
    ($it:expr; @$type:ty; $p:pat => $e:expr, $b1:expr, $b2:expr, $($t:tt)*) => {
        comprehension![$it; @$type; $p => $e, $b1 & $b2, $($t)*]
    };
}

#[cfg(test)]
mod tests {
    use crate::tools::Xorshift;
    #[test]
    fn test_comprehension() {
        use std::collections::{HashMap, HashSet};
        let mut rng = Xorshift::default();
        for _ in 0..1000 {
            let n = rng.random(0..=100);
            let values: Vec<_> = rng.random_iter(-100i32..=100).take(n).collect();
            assert_eq!(
                comprehension!(values.iter().copied(); @HashSet<_>),
                (values.iter().copied()).collect::<HashSet<_>>()
            );
            assert_eq!(comprehension!(values.iter().copied()), values.clone());
            assert_eq!(
                comprehension!(values.iter().copied(); @HashMap<_,_>; i => (i, i + i)),
                (values.iter().copied())
                    .map(|i| (i, i + i))
                    .collect::<HashMap<_, _>>()
            );
            assert_eq!(
                comprehension!(values.iter().copied(); i => i + i),
                (values.iter().copied()).map(|i| i + i).collect::<Vec<_>>()
            );
            assert_eq!(
                comprehension!(values.iter().copied(); &i, i % 2 == 0),
                (values.iter().copied())
                    .filter(|&i| i % 2 == 0)
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                comprehension!(values.iter().copied(); i => i + i, i % 2 == 0),
                (values.iter().copied())
                    .filter_map(|i| if i % 2 == 0 { Some(i + i) } else { None })
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                comprehension!(values.iter().copied(); i => i + i, i % 2 == 0, i % 3 == 0),
                (values.iter().copied())
                    .filter_map(|i| if i % 2 == 0 && i % 3 == 0 {
                        Some(i + i)
                    } else {
                        None
                    })
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                comprehension!(values.iter().copied(); i => i + i, i % 2 == 0, i % 3 == 0, i % 4 == 0),
                (values.iter().copied())
                    .filter_map(|i| if i % 2 == 0 && i % 3 == 0 && i % 4 == 0 {
                        Some(i + i)
                    } else {
                        None
                    })
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                comprehension!(values.iter().copied(); @HashMap<_,_>; i => (i / 24, i), i % 2 == 0, i % 3 == 0, i % 4 == 0),
                (values.iter().copied())
                    .filter_map(|i| if i % 2 == 0 && i % 3 == 0 && i % 4 == 0 {
                        Some((i / 24, i))
                    } else {
                        None
                    })
                    .collect::<HashMap<_, _>>()
            );
        }
    }
}
