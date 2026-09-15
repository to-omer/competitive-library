use competitive::algorithm::CartesianTree;
use competitive::prelude::*;

#[verify::library_checker("cartesian_tree")]
pub fn cartesian_tree(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, a: [i32; n]);
    let ct = CartesianTree::new(&a);
    pp!(@it ct.parents.iter().map(|&p| if p == !0 { ct.root } else { p }));
}
