use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::AdditiveOperation,
    data_structure::{BinaryIndexedTree, FibHashMap},
};

#[verify::library_checker("static_range_count_distinct")]
pub fn static_range_count_distinct(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [u32; n], lr: [(usize, usize); q]);
    let mut order: Vec<_> = (0..q).collect();
    order.sort_unstable_by_key(|&i| lr[i].1);
    let mut bit = BinaryIndexedTree::<AdditiveOperation<i64>>::new(n);
    let mut last = FibHashMap::default();
    let mut ans = vec![0; q];
    let mut r = 0;
    for i in order {
        while r < lr[i].1 {
            if let Some(prev) = last.insert(a[r], r) {
                bit.update(prev, -1);
            }
            bit.update(r, 1);
            r += 1;
        }
        ans[i] = bit.fold(lr[i].0, lr[i].1);
    }
    for ans in ans {
        writeln!(writer, "{}", ans).ok();
    }
}
