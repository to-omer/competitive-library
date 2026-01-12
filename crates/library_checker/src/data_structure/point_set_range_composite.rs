use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::LinearOperation,
    data_structure::SegmentTree,
    num::{MInt, mint_basic::MInt998244353},
};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Set { p: usize, cd: (MInt998244353, MInt998244353) }
        1 => Apply { l: usize, r: usize, x: MInt998244353 }
    }
}

#[verify::library_checker("point_set_range_composite")]
pub fn point_set_range_composite(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, ab: [(MInt998244353, MInt998244353); n]);
    let mut seg = SegmentTree::<LinearOperation<_>>::from_vec(ab);
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Set { p, cd } => {
                seg.set(p, cd);
            }
            Query::Apply { l, r, x } => {
                let (a, b) = seg.fold(l..r);
                writeln!(writer, "{}", a * x + b).ok();
            }
        }
    }
}
