use competitive::data_structure::RangeFrequency;
use competitive::prelude::*;

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Set { k: usize, v: i32 }
        1 => Query { l: usize, r: usize, x: i32 }
    }
}

#[verify::library_checker("point_set_range_frequency")]
pub fn point_set_range_frequency(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [i32; n]);
    let mut rf = RangeFrequency::new(a);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Set { k, v } => {
                rf.set(k, v);
            }
            Query::Query { l, r, x } => {
                rf.query(l, r, x);
            }
        }
    }
    let results = rf.execute();
    pp!(@lf @it results);
}
