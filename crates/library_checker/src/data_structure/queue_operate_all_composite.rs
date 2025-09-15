use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::LinearOperation,
    data_structure::QueueAggregation,
    num::{MInt, mint_basic::MInt998244353},
};

#[verify::library_checker("queue_operate_all_composite")]
pub fn queue_operate_all_composite(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, q);
    let mut que = QueueAggregation::<LinearOperation<_>>::new();
    for _ in 0..q {
        match scanner.scan::<usize>() {
            0 => {
                scan!(scanner, ab: (MInt998244353, MInt998244353));
                que.push(ab);
            }
            1 => {
                que.pop();
            }
            2 => {
                scan!(scanner, x: MInt998244353);
                let (a, b) = que.fold_all();
                writeln!(writer, "{}", a * x + b).ok();
            }
            _ => panic!("unknown query"),
        }
    }
}
