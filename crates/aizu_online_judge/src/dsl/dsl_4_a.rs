use competitive::prelude::*;

#[verify::aizu_online_judge("DSL_4_A")]
pub fn dsl_4_a(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, xyxy: [(i64, i64, i64, i64); n]);
    let (mut xs, mut ys) = (Vec::with_capacity(2 * n), Vec::with_capacity(2 * n));
    xs.extend(xyxy.iter().map(|t| t.0));
    ys.extend(xyxy.iter().map(|t| t.1));
    xs.extend(xyxy.iter().map(|t| t.2));
    ys.extend(xyxy.iter().map(|t| t.3));
    xs.sort_unstable();
    ys.sort_unstable();
    xs.dedup();
    ys.dedup();
    let mut qs = vec![vec![]; xs.len()];
    for (x1, y1, x2, y2) in xyxy {
        let x1 = xs.binary_search(&x1).unwrap_or_else(|x| x);
        let x2 = xs.binary_search(&x2).unwrap_or_else(|x| x);
        let y1 = ys.binary_search(&y1).unwrap_or_else(|x| x);
        let y2 = ys.binary_search(&y2).unwrap_or_else(|x| x);
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
