use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::LinearOperation,
    data_structure::SegmentTree,
    num::{MInt, mint_basic::MInt998244353},
};

#[verify::library_checker("point_set_range_composite")]
pub fn point_set_range_composite(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, ab: [(MInt998244353, MInt998244353); n]);
    let mut seg = SegmentTree::<LinearOperation<_>>::from_vec(ab);
    for _ in 0..q {
        match scanner.scan::<usize>() {
            0 => {
                scan!(scanner, p, cd: (MInt998244353, MInt998244353));
                seg.set(p, cd);
            }
            1 => {
                scan!(scanner, l, r, x: MInt998244353);
                let (a, b) = seg.fold(l..r);
                writeln!(writer, "{}", a * x + b).ok();
            }
            _ => panic!("unknown query"),
        }
    }
}
