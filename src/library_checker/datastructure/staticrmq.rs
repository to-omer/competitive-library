pub use crate::algebra::operations::MinOperation;
pub use crate::data_structure::disjoint_sparse_table::DisjointSparseTable;
pub use crate::data_structure::segment_tree::SegmentTree;
pub use crate::scan;
pub use crate::tools::scanner::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://judge.yosupo.jp/problem/staticrmq")]
pub fn staticrmq_disjoint_sparse_table(
    reader: &mut impl Read,
    writer: &mut impl Write,
) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [u64; n], lr: [(usize, usize); q]);
    let table = DisjointSparseTable::new(a, MinOperation::new());
    for (l, r) in lr.into_iter() {
        writeln!(writer, "{}", table.fold(l, r))?;
    }
    Ok(())
}

#[verify_attr::verify("https://judge.yosupo.jp/problem/staticrmq")]
pub fn staticrmq_segment_tree(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [u64; n], lr: [(usize, usize); q]);
    let seg = SegmentTree::from_vec(a, MinOperation::new());
    for (l, r) in lr.into_iter() {
        writeln!(writer, "{}", seg.fold(l, r))?;
    }
    Ok(())
}
