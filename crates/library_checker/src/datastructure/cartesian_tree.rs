#[doc(no_inline)]
pub use competitive::algorithm::CartesianTree;
use competitive::prelude::*;

#[verify::library_checker("cartesian_tree")]
pub fn cartesian_tree(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, a: [i32; n]);
    let ct = CartesianTree::new(&a);
    iter_print!(writer, @it ct.parents.iter().map(|&p| if p == !0 { ct.root } else { p }) );
}
