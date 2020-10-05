#[snippet::entry]
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
        comprehension![$it; @$type; $p => $e, $b1 & $b2];
    };
    ($it:expr; @$type:ty; $p:pat => $e:expr, $b1:expr, $b2:expr, $($t:tt)*) => {
        comprehension![$it; @$type; $p => $e, $b1 & $b2, $($t)*]
    };
}

#[test]
fn test_comprehension() {
    use std::collections::{HashMap, HashSet};
    const N: usize = 100;
    assert_eq!(
        comprehension!(0..N; @HashSet<_>),
        (0..N).collect::<HashSet<_>>()
    );
    assert_eq!(comprehension!(0..N), (0..N).collect::<Vec<_>>());
    assert_eq!(
        comprehension!(0..N; @HashMap<_,_>; i => (i, i + i)),
        (0..N).map(|i| (i, i + i)).collect::<HashMap<_, _>>()
    );
    assert_eq!(
        comprehension!(0..N; i => i + i),
        (0..N).map(|i| i + i).collect::<Vec<_>>()
    );
    assert_eq!(
        comprehension!(0..N; &i, i % 2 == 0),
        (0..N).filter(|&i| i % 2 == 0).collect::<Vec<_>>()
    );
    assert_eq!(
        comprehension!(0..N; i => i + i, i % 2 == 0),
        (0..N)
            .filter_map(|i| if i % 2 == 0 { Some(i + i) } else { None })
            .collect::<Vec<_>>()
    );
    assert_eq!(
        comprehension!(0..N; i => i + i, i % 2 == 0, i % 3 == 0),
        (0..N)
            .filter_map(|i| if i % 2 == 0 && i % 3 == 0 {
                Some(i + i)
            } else {
                None
            })
            .collect::<Vec<_>>()
    );
    assert_eq!(
        comprehension!(0..N; i => i + i, i % 2 == 0, i % 3 == 0, i % 4 == 0),
        (0..N)
            .filter_map(|i| if i % 2 == 0 && i % 3 == 0 && i % 4 == 0 {
                Some(i + i)
            } else {
                None
            })
            .collect::<Vec<_>>()
    );
    assert_eq!(
        comprehension!(0..N; @HashMap<_,_>; i => (i / 24, i), i % 2 == 0, i % 3 == 0, i % 4 == 0),
        (0..N)
            .filter_map(|i| if i % 2 == 0 && i % 3 == 0 && i % 4 == 0 {
                Some((i / 24, i))
            } else {
                None
            })
            .collect::<HashMap<_, _>>()
    );
}
