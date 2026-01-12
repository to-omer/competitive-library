use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::LinearOperation,
    data_structure::QueueAggregation,
    num::{MInt, mint_basic::MInt998244353},
};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Push { ab: (MInt998244353, MInt998244353) }
        1 => Pop
        2 => Apply { x: MInt998244353 }
    }
}

#[verify::library_checker("queue_operate_all_composite")]
pub fn queue_operate_all_composite(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, q);
    let mut que = QueueAggregation::<LinearOperation<_>>::new();
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Push { ab } => {
                que.push(ab);
            }
            Query::Pop => {
                que.pop();
            }
            Query::Apply { x } => {
                let (a, b) = que.fold_all();
                writeln!(writer, "{}", a * x + b).ok();
            }
        }
    }
}
