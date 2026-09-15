use competitive::data_structure::{RangeFrequency, WaveletMatrix};
use competitive::prelude::*;

#[verify::library_checker("static_range_frequency")]
pub fn static_range_frequency(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [u32; n]);
    let mut range_frequency = RangeFrequency::new(a);
    for _ in 0..q {
        sc!(l, r, x: u32);
        range_frequency.query(l, r, x);
    }
    pp!(@lf @it range_frequency.execute());
}

#[verify::library_checker("static_range_frequency")]
pub fn static_range_frequency_wavelet_matrix(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [usize; n]);
    let wm = WaveletMatrix::new(a);
    for _ in 0..q {
        sc!(l, r, x: usize);
        let ans = wm.rank(x, l..r);
        pp!(ans);
    }
}
