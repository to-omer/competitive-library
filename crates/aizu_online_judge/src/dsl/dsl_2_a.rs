use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::MinOperation, data_structure::SegmentTree};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Update { x: usize, y: usize }
        1 => Fold { x: usize, y: usize }
    }
}

#[verify::aizu_online_judge("DSL_2_A")]
pub fn dsl_2_a(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut seg = SegmentTree::<MinOperation<_>>::new(n);
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Update { x, y } => {
                seg.set(x, y as i32);
            }
            Query::Fold { x, y } => {
                writeln!(writer, "{}", seg.fold(x..=y)).ok();
            }
        }
    }
}
