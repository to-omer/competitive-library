use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::FindMajorityOperation,
    data_structure::{RangeFrequency, SegmentTree},
};

#[verify::library_checker("majority_voting")]
pub fn majority_voting(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [i32; n]);
    let mut seg = SegmentTree::<FindMajorityOperation<i32>>::from_vec(
        a.iter().map(|&a| (Some(a), 1)).collect(),
    );
    let mut rf = RangeFrequency::new(a);
    let mut out = vec![];
    for _ in 0..q {
        scan!(scanner, ty);
        if ty == 0 {
            scan!(scanner, p, x: i32);
            seg.set(p, (Some(x), 1));
            rf.set(p, x);
        } else {
            scan!(scanner, l, r);
            let x = seg.fold(l..r).0.unwrap_or(-1);
            out.push((x, r - l));
            rf.query(l, r, x);
        }
    }
    rf.execute_with_callback(|i, v| {
        if out[i].1 >= 2 * v {
            out[i].0 = -1;
        }
    });
    iter_print!(writer, @lf @it out.iter().map(|&(x, _)| x));
}
