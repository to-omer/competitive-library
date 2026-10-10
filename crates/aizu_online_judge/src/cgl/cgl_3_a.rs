use competitive::{geometry::polygon_area2, num::Complex, prelude::*};

#[verify::aizu_online_judge("CGL_3_A")]
pub fn cgl_3_a(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, ps: [Complex<i64>; n]);
    pp!(@fmt ("{:.1}", polygon_area2(&ps) as f64 / 2.0));
}
