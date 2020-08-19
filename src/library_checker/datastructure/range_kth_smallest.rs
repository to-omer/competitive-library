pub use crate::data_structure::WaveletMatrix;
use crate::prelude::*;

#[verify_attr::verify("https://judge.yosupo.jp/problem/range_kth_smallest")]
pub fn range_kth_smallest(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [usize; n]);
    let wm = WaveletMatrix::new(a, 30);
    for (l, r, k) in scanner.iter::<(usize, usize, usize)>().take(q) {
        writeln!(writer, "{}", wm.quantile(l..r, k)).ok();
    }
}
