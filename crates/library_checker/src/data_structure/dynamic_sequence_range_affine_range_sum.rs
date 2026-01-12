use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::RangeSumRangeLinear, data_structure::SplaySequence, num::mint_basic::MInt998244353,
};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Insert { i: usize, x: MInt998244353 }
        1 => Remove { i: usize }
        2 => Reverse { l: usize, r: usize }
        3 => Update { l: usize, r: usize, bc: (MInt998244353, MInt998244353) }
        4 => Fold { l: usize, r: usize }
    }
}

#[verify::library_checker("dynamic_sequence_range_affine_range_sum")]
pub fn dynamic_sequence_range_affine_range_sum(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [MInt998244353; n]);

    let mut seq = SplaySequence::<RangeSumRangeLinear<MInt998244353>>::with_capacity(n + q);
    seq.extend(a);
    for _ in 0..q {
        scan!(scanner, query: Query);
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
                writeln!(writer, "{}", seq.fold(l..r).0).ok();
            }
        }
    }
}
