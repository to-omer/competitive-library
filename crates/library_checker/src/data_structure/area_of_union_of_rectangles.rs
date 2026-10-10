use competitive::geometry;
use competitive::prelude::*;

#[verify::library_checker("area_of_union_of_rectangles")]
pub fn area_of_union_of_rectangles(reader: impl Read, writer: impl Write) {
    prepare_io!(buffered; reader, writer);
    sc!(n, rectangles: [(u32, u32, u32, u32); n]);
    pp!(geometry::area_of_union_of_rectangles(&rectangles));
}
