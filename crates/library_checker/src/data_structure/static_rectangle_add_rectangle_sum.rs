use competitive::prelude::*;
use competitive::{
    algebra::AddMulOperation, geometry::StaticRectangleAddRectangleSum,
    num::mint_basic::MInt998244353 as M,
};

#[verify::library_checker("static_rectangle_add_rectangle_sum")]
pub fn static_rectangle_add_rectangle_sum(reader: impl Read, writer: impl Write) {
    prepare_io!(buffered; reader, writer);
    sc!(n, q, rectangles: [(u32, u32, u32, u32, M); iter n]);
    let mut sums = StaticRectangleAddRectangleSum::<AddMulOperation<M>>::new(rectangles);
    for _ in 0..q {
        sc!(l: u32, d: u32, r: u32, u: u32);
        sums.query(l, d, r, u);
    }
    pp!(@lf @it sums.execute());
}
