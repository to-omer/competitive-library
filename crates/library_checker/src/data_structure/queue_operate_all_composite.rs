use competitive::prelude::*;
use competitive::{
    algebra::LinearOperation, data_structure::QueueAggregation, num::mint_basic::MInt998244353 as M,
};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Push { ab: (M, M) }
        1 => Pop
        2 => Apply { x: M }
    }
}

#[verify::library_checker("queue_operate_all_composite")]
pub fn queue_operate_all_composite(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(q);
    let mut que = QueueAggregation::<LinearOperation<_>>::new();
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Push { ab } => {
                que.push(ab);
            }
            Query::Pop => {
                que.pop();
            }
            Query::Apply { x } => {
                let (a, b) = que.fold_all();
                pp!(a * x + b);
            }
        }
    }
}
