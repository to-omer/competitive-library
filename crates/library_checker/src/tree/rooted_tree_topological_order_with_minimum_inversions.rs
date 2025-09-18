#[doc(no_inline)]
pub use competitive::algorithm::solve_01_on_tree;
use competitive::prelude::*;

#[verify::library_checker("rooted_tree_topological_order_with_minimum_inversions")]
pub fn rooted_tree_topological_order_with_minimum_inversions(
    reader: impl Read,
    mut writer: impl Write,
) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, p: [usize; n - 1], c: [usize; n], d: [usize; n]);
    let (cost, ord) = solve_01_on_tree(n, |u| (c[u], d[u]), 0, |u| p[u - 1]);
    iter_print!(writer, cost; @it ord);
}
