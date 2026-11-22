use competitive::data_structure::RangeLinearAddRangeMin;
use competitive::prelude::*;

#[codesnip::entry("Query")]
competitive::define_enum_scan! {
    enum Query: u8 {
        0 => Add { l: usize, r: usize, b: i64, c: i64 }
        1 => Min { l: usize, r: usize }
    }
}

#[verify::library_checker(
    "range_linear_add_range_min",
    include("RangeLinearAddRangeMin", "Query")
)]
pub fn range_linear_add_range_min(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [i64; iter n]);
    let mut data = RangeLinearAddRangeMin::from_keys(a);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Add { l, r, b, c } => data.update(l..r, b, c),
            Query::Min { l, r } => {
                pp!(data.fold(l..r));
            }
        }
    }
}
