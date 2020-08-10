pub use crate::algorithm::Compress;
use crate::scan;
use crate::tools::{read_all, Scanner};
use std::io::{Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/4/DSL_4_A")]
pub fn dsl_4_a(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, xyxy: [(i64, i64, i64, i64); n]);
    let xs: Compress<_> = (0..2 * n)
        .map(|i| {
            if i % 2 == 0 {
                xyxy[i / 2].0
            } else {
                xyxy[i / 2].2
            }
        })
        .collect();
    let ys: Compress<_> = (0..2 * n)
        .map(|i| {
            if i % 2 == 0 {
                xyxy[i / 2].1
            } else {
                xyxy[i / 2].3
            }
        })
        .collect();
    let mut qs = vec![vec![]; xs.len()];
    for (x1, y1, x2, y2) in xyxy {
        let x1 = xs.get(&x1);
        let x2 = xs.get(&x2);
        let y1 = ys.get(&y1);
        let y2 = ys.get(&y2);
        qs[x1].push((y1, 1));
        qs[x1].push((y2, -1));
        qs[x2].push((y1, -1));
        qs[x2].push((y2, 1));
    }
    let mut ans = 0;
    let mut acc = vec![0; ys.len()];
    for i in 0..xs.len() - 1 {
        let d = xs[i + 1] - xs[i];
        let mut tmp = vec![0; ys.len()];
        for &(j, c) in qs[i].iter() {
            tmp[j] += c;
        }
        for j in 0..ys.len() - 1 {
            tmp[j + 1] += tmp[j];
        }
        for (acc, tmp) in acc.iter_mut().zip(tmp.iter_mut()) {
            *acc += *tmp;
        }
        for j in 0..ys.len() - 1 {
            if acc[j] > 0 {
                ans += d * (ys[j + 1] - ys[j]);
            }
        }
    }
    writeln!(writer, "{}", ans).ok();
}
