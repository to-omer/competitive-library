#[doc(no_inline)]
pub use competitive::data_structure::WaveletMatrix;
use competitive::prelude::*;

#[verify::library_checker("static_range_frequency")]
pub fn static_range_frequency(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [usize; n]);
    let wm = WaveletMatrix::new(a, 30);
    for _ in 0..q {
        scan!(scanner, l, r, x: usize);
        let ans = wm.rank(x, l..r);
        iter_print!(writer, ans);
    }
}
