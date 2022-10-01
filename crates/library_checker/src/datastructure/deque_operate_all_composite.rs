use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::LinearOperation,
    data_structure::DequeAggregation,
    num::{mint_basic::MInt998244353, MInt},
};

#[verify::library_checker("deque_operate_all_composite")]
pub fn deque_operate_all_composite(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, q);
    let mut deq = DequeAggregation::<LinearOperation<_>>::new();
    for _ in 0..q {
        match scanner.scan::<usize>() {
            0 => {
                scan!(scanner, ab: (MInt998244353, MInt998244353));
                deq.push_front(ab);
            }
            1 => {
                scan!(scanner, ab: (MInt998244353, MInt998244353));
                deq.push_back(ab);
            }
            2 => {
                deq.pop_front();
            }
            3 => {
                deq.pop_back();
            }
            4 => {
                scan!(scanner, x: MInt998244353);
                let (a, b) = deq.fold_all();
                writeln!(writer, "{}", a * x + b).ok();
            }
            _ => panic!("unknown query"),
        }
    }
}
