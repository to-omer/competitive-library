use competitive::prelude::*;
use competitive::{
    algebra::{LinearOperation, RangeProductRangeUpdate},
    data_structure::{DaryAssignCompositeKernel, DaryLazySegmentTree, LazySegmentTree},
    num::mint_basic::MInt998244353 as M,
};

#[verify::library_checker(
    "range_set_range_composite",
    include("LazySegmentTree", "LinearOperation", "MInt")
)]
pub fn range_set_range_composite(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, functions: [(M, M); iter n]);
    let mut seg =
        LazySegmentTree::<RangeProductRangeUpdate<LinearOperation<M>>>::from_keys(functions);
    for _ in 0..q {
        sc!(t: u8, l, r);
        if t == 0 {
            sc!(a: M, b: M);
            seg.update(l..r, Some((a, b)));
        } else {
            sc!(x: M);
            let ((a, b), _) = seg.fold(l..r);
            pp!(a * x + b);
        }
    }
}

#[verify::library_checker(
    "range_set_range_composite",
    include("DaryAssignCompositeKernel"),
    huge_pages
)]
pub fn range_set_range_composite_dary(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, input: [u32; 2 * n + 4 * q; capacity 2 * n + 4 * q]);
    let (functions, tokens) = input.split_at(2 * n);
    let (functions, _): (&[[u32; 2]], _) = functions.as_chunks();
    let mut seg = DaryLazySegmentTree::<DaryAssignCompositeKernel>::from_keys(
        functions
            .iter()
            .map(|&[a, b]| (M::new_unchecked(a), M::new_unchecked(b))),
    );
    let mut tokens = tokens
        .iter()
        .copied()
        .chain(std::iter::repeat_with(|| sv!(u32)));
    let queries: Vec<_> = (0..q)
        .map(|_| {
            let [t, l, r, x] = std::array::from_fn(|_| tokens.next().unwrap());
            let y = if t == 0 { tokens.next().unwrap() } else { 0 };
            [t, l, r, x, y]
        })
        .collect();
    drop(input);
    for (i, &[t, l, r, x, y]) in queries.iter().enumerate() {
        if let Some(&[_, l, r, ..]) = queries.get(i + 4) {
            seg.prefetch(l as usize);
            seg.prefetch(r as usize - 1);
        }
        if t == 0 {
            seg.update(
                l as usize..r as usize,
                (M::new_unchecked(x), M::new_unchecked(y)),
            );
        } else {
            let (a, b) = seg.fold(l as usize..r as usize);
            pp!(a * M::new_unchecked(x) + b);
        }
    }
}
