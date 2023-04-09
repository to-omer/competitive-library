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
    [|$i:pat| $e:expr; $len:expr] => {
        $crate::array![@inner data = [data.iter_mut().enumerate().for_each(|($i, item)| *item = MaybeUninit::new($e)); $len]]
    };
    [$e:expr; $len:expr] => {{
        let e = $e;
        $crate::array![|| Clone::clone(&e); $len]
    }};
}

#[test]
fn test_array() {
    let mut x = 0;
    assert_eq!(array![1; 3], [1; 3]);
    assert_eq!(array![|| { x += 1; x }; 3], [1, 2, 3]);
    assert_eq!(array![|i| i + 1; 3], [1, 2, 3]);
}
