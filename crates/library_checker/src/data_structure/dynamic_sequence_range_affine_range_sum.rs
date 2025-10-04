use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::RangeSumRangeLinear, data_structure::SplaySequence, num::mint_basic::MInt998244353,
};

#[verify::library_checker("dynamic_sequence_range_affine_range_sum")]
pub fn dynamic_sequence_range_affine_range_sum(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [MInt998244353; n]);

    let mut seq = SplaySequence::<RangeSumRangeLinear<MInt998244353>>::with_capacity(n + q);
    seq.extend(a);
    for _ in 0..q {
        match scanner.scan::<usize>() {
            0 => {
                scan!(scanner, i, x: MInt998244353);
                seq.insert(i, x);
            }
            1 => {
                scan!(scanner, i);
                seq.remove(i);
            }
            2 => {
                scan!(scanner, l, r);
                seq.reverse(l..r);
            }
            3 => {
                scan!(scanner, l, r, bc: (MInt998244353, MInt998244353));
                seq.update(l..r, bc);
            }
            4 => {
                scan!(scanner, l, r);
                writeln!(writer, "{}", seq.fold(l..r).0).ok();
            }
            _ => unreachable!("unknown query"),
        }
    }
}
