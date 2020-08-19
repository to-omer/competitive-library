use crate::prelude::*;

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/5/DSL_5_B")]
pub fn dsl_5_b(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, xyxy: [(usize, usize, usize, usize); n]);
    let mut acc = vec![vec![0; 1001]; 1001];
    for (x1, y1, x2, y2) in xyxy {
        acc[x1][y1] += 1;
        acc[x2][y1] -= 1;
        acc[x1][y2] -= 1;
        acc[x2][y2] += 1;
    }
    for i in 0..=1000 {
        for j in 0..1000 {
            acc[i][j + 1] += acc[i][j];
        }
    }
    for i in 0..1000 {
        for j in 0..=1000 {
            acc[i + 1][j] += acc[i][j];
        }
    }
    writeln!(
        writer,
        "{}",
        acc.into_iter()
            .map(|acc| acc.into_iter().max().unwrap_or_default())
            .max()
            .unwrap_or_default()
    )
    .ok();
}
