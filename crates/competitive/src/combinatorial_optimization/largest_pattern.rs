pub fn largest_square(h: usize, w: usize, ok: impl Fn(usize, usize) -> bool) -> usize {
    let mut dp = vec![vec![0usize; w + 1]; h + 1];
    for i in 0..h {
        for j in 0..w {
            if ok(i, j) {
                dp[i + 1][j + 1] = dp[i][j].min(dp[i + 1][j]).min(dp[i][j + 1]) + 1;
            }
        }
    }
    dp.into_iter()
        .map(|d| d.into_iter().max().unwrap_or_default())
        .max()
        .unwrap_or_default()
        .pow(2)
}

pub fn largest_rectangle(hist: &[usize]) -> usize {
    let mut stack = Vec::<(_, _)>::new();
    let mut res = 0;
    for (i, h) in hist.iter().cloned().enumerate() {
        let mut j = i;
        while stack.last().is_some_and(|x| x.1 > h) {
            let (k, p) = stack.pop().unwrap();
            res = res.max((i - k) * p);
            j = k;
        }
        if stack.last().is_none_or(|x| x.1 < h) {
            stack.push((j, h));
        }
    }
    while let Some((i, h)) = stack.pop() {
        res = res.max((hist.len() - i) * h);
    }
    res
}

pub fn largest_rectangle_in_grid(h: usize, w: usize, ok: impl Fn(usize, usize) -> bool) -> usize {
    let mut hist = vec![0; w];
    let mut res = 0;
    for i in 0..h {
        for (j, hist) in hist.iter_mut().enumerate() {
            *hist = if ok(i, j) { *hist + 1 } else { 0 };
        }
        res = res.max(largest_rectangle(&hist));
    }
    res
}
