use competitive::prelude::*;
use competitive::{
    algebra::AdditiveOperation,
    algorithm::SliceSortExt,
    data_structure::{CompressedSegmentTree2d, WaveletMatrix, WaveletMatrixPointAdd},
};

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
        .map(|&(x, y, w)| (x, y, w as i64))
        .chain(queries.iter().filter_map(|&query| match query {
            Query::Add { x, y, .. } => Some((x, y, 0)),
            Query::Sum { .. } => None,
        }))
        .collect();
    let mut order: Vec<_> = (0..points.len()).collect();
    order.radix_sort_by_key(|&i| points[i].0);
    let mut positions = vec![0; points.len()];
    let mut xs = Vec::with_capacity(points.len());
    let mut ys = Vec::with_capacity(points.len());
    let mut weights = Vec::with_capacity(points.len());
    for (i, &point) in order.iter().enumerate() {
        positions[point] = i;
        let (x, y, w) = points[point];
        xs.push(x);
        ys.push(y);
        weights.push(w);
    }
    let wm = WaveletMatrix::new(ys);
    let mut fold: WaveletMatrixPointAdd<_, AdditiveOperation<i64>> = wm.build_point_add(&weights);

    let mut point = n;
    for query in queries {
        match query {
            Query::Add { w, .. } => {
                fold.update(positions[point], w as i64);
                point += 1;
            }
            Query::Sum { l, d, r, u } => {
                let l = xs.partition_point(|&x| x < l);
                let r = xs.partition_point(|&x| x < r);
                writeln!(writer, "{}", fold.fold_range(d..u, l..r)).ok();
            }
        }
    }
}

#[verify::library_checker("point_add_rectangle_sum")]
pub fn point_add_rectangle_sum_compressed_segment_tree(reader: impl Read, mut writer: impl Write) {
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
