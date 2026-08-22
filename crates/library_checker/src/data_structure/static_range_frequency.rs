use competitive::data_structure::{RangeFrequency, WaveletMatrix};
use competitive::prelude::*;

#[verify::library_checker("static_range_frequency")]
pub fn static_range_frequency(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [u32; n]);
    let mut range_frequency = RangeFrequency::new(a);
    for _ in 0..q {
        scan!(scanner, l, r, x: u32);
        range_frequency.query(l, r, x);
    }
    iter_print!(writer, @lf @it range_frequency.execute());
}

#[verify::library_checker("static_range_frequency")]
pub fn static_range_frequency_wavelet_matrix(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [usize; n]);
    let wm = WaveletMatrix::new(a);
    for _ in 0..q {
        scan!(scanner, l, r, x: usize);
        let ans = wm.rank(x, l..r);
        iter_print!(writer, ans);
    }
}
