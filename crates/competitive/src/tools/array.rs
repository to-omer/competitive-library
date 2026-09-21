#[macro_export]
macro_rules! array {
    [@inner $data:ident = [$init:expr; $len:expr]] => {{
        use ::std::mem::{ManuallyDrop, MaybeUninit};
        let mut $data: [MaybeUninit<_>; $len] = unsafe { MaybeUninit::uninit().assume_init() };
        $init;
        #[repr(C)]
        union __Transmuter<const N: usize, T: Clone> {
            src: ManuallyDrop<[MaybeUninit<T>; N]>,
            dst: ManuallyDrop<[T; N]>,
        }
        ManuallyDrop::into_inner(unsafe { __Transmuter { src: ManuallyDrop::new($data) }.dst })
    }};
    [|| $e:expr; $len:expr] => {
        $crate::array![@inner data = [data.iter_mut().for_each(|item| *item = MaybeUninit::new($e)); $len]]
    };
    [|$i:pat_param| $e:expr; $len:expr] => {
        $crate::array![@inner data = [data.iter_mut().enumerate().for_each(|($i, item)| *item = MaybeUninit::new($e)); $len]]
    };
    [$e:expr; $len:expr] => {{
        let e = $e;
        $crate::array![|| Clone::clone(&e); $len]
    }};
}

#[test]
fn test_array() {
    use crate::tools::Xorshift;
    use std::array;
    fn check<const N: usize>(start: i32, step: i32) {
        let mut x = start;
        assert_eq!(array![start; N], [start; N]);
        assert_eq!(
            array![|| { x += step; x }; N],
            array::from_fn(|i| start + (i as i32 + 1) * step)
        );
        assert_eq!(x, start + N as i32 * step);
        assert_eq!(
            array![|i| start + i as i32 * step; N],
            array::from_fn(|i| start + i as i32 * step)
        );
    }
    let mut rng = Xorshift::default();
    for (start, step) in (-5..=5)
        .flat_map(|start| (-5..=5).map(move |step| (start, step)))
        .chain(rng.random_iter((-1000..=1000, -1000..=1000)).take(1000))
    {
        check::<0>(start, step);
        check::<1>(start, step);
        check::<2>(start, step);
        check::<3>(start, step);
        check::<4>(start, step);
        check::<8>(start, step);
        check::<16>(start, step);
        check::<32>(start, step);
    }
}
