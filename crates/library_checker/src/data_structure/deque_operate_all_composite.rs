use competitive::prelude::*;
use competitive::{
    algebra::LinearOperation, data_structure::DequeAggregation, num::mint_basic::MInt998244353 as M,
};

#[codesnip::entry("Query", include("MInt"))]
competitive::define_enum_scan! {
    enum Query: usize {
        0 => PushFront { ab: (M, M) }
        1 => PushBack { ab: (M, M) }
        2 => PopFront
        3 => PopBack
        4 => Apply { x: M }
    }
}

#[verify::library_checker(
    "deque_operate_all_composite",
    include("LinearOperation", "Query", "sliding_window_aggregation")
)]
pub fn deque_operate_all_composite(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(q);
    let mut deq = DequeAggregation::<LinearOperation<_>>::new();
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::PushFront { ab } => {
                deq.push_front(ab);
            }
            Query::PushBack { ab } => {
                deq.push_back(ab);
            }
            Query::PopFront => {
                deq.pop_front();
            }
            Query::PopBack => {
                deq.pop_back();
            }
            Query::Apply { x } => {
                let (a, b) = deq.fold_all();
                pp!(a * x + b);
            }
        }
    }
}
