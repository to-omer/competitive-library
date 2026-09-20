use competitive::prelude::*;
use competitive::{data_structure::BitSet, math::BitMatrix};

#[verify::library_checker("matrix_rank_mod_2")]
pub fn matrix_rank_mod_2(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m);
    // Transpose very tall matrices to avoid allocating millions of short rows.
    let transpose = n / 2 > m;
    let mut a = BitMatrix::zeros(if transpose { (m, n) } else { (n, m) });
    if transpose && m >= 64 {
        let mut words = vec![0u64; m];
        for first in (0..n).step_by(64) {
            words.fill(0);
            for i in 0..64.min(n - first) {
                sc!(row: &str);
                for (word, b) in words.iter_mut().zip(row.bytes()) {
                    *word |= u64::from(b == b'1') << i;
                }
            }
            for (row, &word) in a.data.iter_mut().zip(&words) {
                row.words_mut()[first / 64] = word;
            }
        }
    } else {
        for i in 0..if m == 0 { 0 } else { n } {
            sc!(row: &str);
            if !transpose {
                a.data[i] = BitSet::from_binary(row).unwrap();
            } else {
                for (j, b) in row.bytes().enumerate() {
                    a[j].words_mut()[i / 64] |= u64::from(b == b'1') << (i % 64);
                }
            }
            if transpose
                && i == 63
                && BitMatrix::new_with((m, 64), |row, col| a[row].get(col)).rank() == m
            {
                pp!(m);
                return;
            }
        }
    }
    pp!(a.rank());
}
