use competitive::prelude::*;
use competitive::{
    algebra::RangeSumRangeAdd,
    data_structure::{ImplicitSplayTree, ImplicitTreap},
};

competitive::define_enum_scan! {
    enum Query: u8 {
        0 => Reverse { l: usize, r: usize }
        1 => Sum { l: usize, r: usize }
    }
}

#[verify::library_checker("range_reverse_range_sum")]
pub fn range_reverse_range_sum(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [i64; n]);
    let mut seq = ImplicitTreap::<RangeSumRangeAdd<i64>>::with_capacity(n);
    seq.extend(a);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Reverse { l, r } => {
                seq.reverse(l..r);
            }
            Query::Sum { l, r } => {
                let ans = seq.fold(l..r).0;
                pp!(ans);
            }
        }
    }
}

#[verify::library_checker("range_reverse_range_sum")]
pub fn range_reverse_range_sum_implicit_splay_tree(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [i64; n]);
    let mut seq = ImplicitSplayTree::<RangeSumRangeAdd<i64>>::with_capacity(n);
    seq.extend(a);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Reverse { l, r } => {
                seq.reverse(l..r);
            }
            Query::Sum { l, r } => {
                let ans = seq.fold(l..r).0;
                pp!(ans);
            }
        }
    }
}
