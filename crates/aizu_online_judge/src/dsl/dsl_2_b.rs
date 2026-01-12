use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::AdditiveOperation, data_structure::SegmentTree};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Update { x: Usize1, y: usize }
        1 => Fold { x: Usize1, y: usize }
    }
}

#[verify::aizu_online_judge("DSL_2_B")]
pub fn dsl_2_b(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut seg = SegmentTree::<AdditiveOperation<_>>::new(n);
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Update { x, y } => {
                seg.update(x, y as i32);
            }
            Query::Fold { x, y } => {
                writeln!(writer, "{}", seg.fold(x..y)).ok();
            }
        }
    }
}
