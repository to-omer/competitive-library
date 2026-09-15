use competitive::prelude::*;
use competitive::{
    algebra::FindMajorityOperation,
    data_structure::{RangeFrequency, SegmentTree},
};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Update { p: usize, x: i32 }
        1 => Query { l: usize, r: usize }
    }
}

#[verify::library_checker("majority_voting")]
pub fn majority_voting(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [i32; n]);
    let mut seg = SegmentTree::<FindMajorityOperation<i32>>::from_vec(
        a.iter().map(|&a| (Some(a), 1)).collect(),
    );
    let mut rf = RangeFrequency::new(a);
    let mut out = vec![];
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Update { p, x } => {
                seg.set(p, (Some(x), 1));
                rf.set(p, x);
            }
            Query::Query { l, r } => {
                let x = seg.fold(l..r).0.unwrap_or(-1);
                out.push((x, r - l));
                rf.query(l, r, x);
            }
        }
    }
    rf.execute_with_callback(|i, v| {
        if out[i].1 >= 2 * v {
            out[i].0 = -1;
        }
    });
    pp!(@lf @it out.iter().map(|&(x, _)| x));
}
