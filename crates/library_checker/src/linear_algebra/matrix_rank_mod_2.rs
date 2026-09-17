use competitive::prelude::*;
use competitive::{data_structure::BitSet, math::BitMatrix};

#[verify::library_checker("matrix_rank_mod_2")]
pub fn matrix_rank_mod_2(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m);
    // Transpose very tall matrices to avoid allocating millions of short rows.
    let transpose = n / 2 > m;
    let mut a = BitMatrix::zeros(if transpose { (m, n) } else { (n, m) });
    for i in 0..if m == 0 { 0 } else { n } {
        sc!(row: &str);
        if !transpose {
            a.data[i] = BitSet::from_binary(row).unwrap();
        } else {
            for (j, b) in row.bytes().enumerate() {
                a[j].set(i, b == b'1');
            }
        }
    }
    pp!(a.rank());
}
