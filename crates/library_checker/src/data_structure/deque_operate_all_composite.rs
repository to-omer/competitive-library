use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::LinearOperation,
    data_structure::DequeAggregation,
    num::{MInt, mint_basic::MInt998244353},
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
pub fn deque_operate_all_composite(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, q);
    let mut deq = DequeAggregation::<LinearOperation<_>>::new();
    for _ in 0..q {
        scan!(scanner, query: Query);
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
                writeln!(writer, "{}", a * x + b).ok();
            }
        }
    }
}
