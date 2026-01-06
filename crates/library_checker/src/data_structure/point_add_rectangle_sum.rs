use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::AdditiveOperation, data_structure::CompressedSegmentTree2d};

competitive::define_enum_scan! {
    #[derive(Clone, Copy)]
    enum Query: u8 {
        0 => Add { x: u32, y: u32, w: u64 }
        1 => Sum { l: u32, d: u32, r: u32, u: u32 }
    }
}

#[verify::library_checker("point_add_rectangle_sum")]
pub fn point_add_rectangle_sum(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, xyw: [(u32, u32, u64); n], queries: [Query; q]);
    let points: Vec<_> = xyw
        .iter()
        .map(|&(x, y, _)| (x, (y,)))
        .chain(queries.iter().filter_map(|&query| {
            if let Query::Add { x, y, .. } = query {
                Some((x, (y,)))
            } else {
                None
            }
        }))
        .collect();

    let mut seg = CompressedSegmentTree2d::<AdditiveOperation<u64>, _, _>::new(&points);
    for &(x, y, w) in &xyw {
        seg.update(&(x, (y,)), &w);
    }

    for query in queries {
        match query {
            Query::Add { x, y, w } => {
                seg.update(&(x, (y,)), &w);
            }
            Query::Sum { l, d, r, u } => {
                let ans = seg.fold(&(l..r, (d..u,)));
                writeln!(writer, "{}", ans).ok();
            }
        }
    }
}
