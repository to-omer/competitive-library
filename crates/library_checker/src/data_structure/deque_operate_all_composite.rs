use competitive::prelude::*;
use competitive::{
    algebra::LinearOperation, data_structure::DequeAggregation, num::mint_basic::MInt998244353,
};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => PushFront { ab: (MInt998244353, MInt998244353) }
        1 => PushBack { ab: (MInt998244353, MInt998244353) }
        2 => PopFront
        3 => PopBack
        4 => Apply { x: MInt998244353 }
    }
}

#[verify::library_checker("deque_operate_all_composite")]
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
