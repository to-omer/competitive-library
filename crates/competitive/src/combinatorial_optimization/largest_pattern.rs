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
        while let Some((k, p)) = stack.pop_if(|x| x.1 > h) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_largest_square() {
        for mask in 0u16..1 << 12 {
            let cell = |i, j| mask & (1u16 << (i * 4 + j)) != 0;
            let mut expected = 0;
            for top in 0..3 {
                for left in 0..4 {
                    for size in 1..=(3 - top).min(4 - left) {
                        if (top..top + size).all(|i| (left..left + size).all(|j| cell(i, j))) {
                            expected = expected.max(size * size);
                        }
                    }
                }
            }
            assert_eq!(largest_square(3, 4, cell), expected);
        }
        for (h, w) in [(0, 4), (3, 0)] {
            assert_eq!(largest_square(h, w, |_, _| true), 0);
        }
    }
}
