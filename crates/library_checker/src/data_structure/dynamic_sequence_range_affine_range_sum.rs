use competitive::prelude::*;
use competitive::{
    algebra::RangeSumRangeLinear,
    data_structure::{ImplicitSplayTree, ImplicitTreap},
    num::mint_basic::MInt998244353 as M,
};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Insert { i: usize, x: M }
        1 => Remove { i: usize }
        2 => Reverse { l: usize, r: usize }
        3 => Update { l: usize, r: usize, bc: (M, M) }
        4 => Fold { l: usize, r: usize }
    }
}

#[verify::library_checker("dynamic_sequence_range_affine_range_sum")]
pub fn dynamic_sequence_range_affine_range_sum(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [M; iter n]);

    let mut seq = ImplicitTreap::<RangeSumRangeLinear<M>>::with_capacity(n + q);
    seq.extend(a);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Insert { i, x } => {
                seq.insert(i, x);
            }
            Query::Remove { i } => {
                seq.remove(i);
            }
            Query::Reverse { l, r } => {
                seq.reverse(l..r);
            }
            Query::Update { l, r, bc } => {
                seq.update(l..r, bc);
            }
            Query::Fold { l, r } => {
                pp!(seq.fold(l..r).0);
            }
        }
    }
}

#[verify::library_checker("dynamic_sequence_range_affine_range_sum")]
pub fn dynamic_sequence_range_affine_range_sum_implicit_splay_tree(
    reader: impl Read,
    writer: impl Write,
) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [M; iter n]);

    let mut seq = ImplicitSplayTree::<RangeSumRangeLinear<M>>::with_capacity(n + q);
    seq.extend(a);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Insert { i, x } => {
                seq.insert(i, x);
            }
            Query::Remove { i } => {
                seq.remove(i);
            }
            Query::Reverse { l, r } => {
                seq.reverse(l..r);
            }
            Query::Update { l, r, bc } => {
                seq.update(l..r, bc);
            }
            Query::Fold { l, r } => {
                pp!(seq.fold(l..r).0);
            }
        }
    }
}
