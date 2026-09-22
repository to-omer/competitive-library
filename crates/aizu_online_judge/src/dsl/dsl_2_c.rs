use competitive::data_structure::Static2DTree;
use competitive::prelude::*;

#[verify::aizu_online_judge("DSL_2_C")]
pub fn dsl_2_c(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, xy: [(i64, i64); iter n]);
    let tree = Static2DTree::new(xy.enumerate().map(|(i, (x, y))| (x, y, i)));
    sc!(q, query: [(i64, i64, i64, i64); iter q]);
    for (sx, tx, sy, ty) in query {
        let mut v = tree.range(sx..tx + 1, sy..ty + 1);
        v.sort();
        for v in v {
            pp!(v);
        }
        pp!();
    }
}
