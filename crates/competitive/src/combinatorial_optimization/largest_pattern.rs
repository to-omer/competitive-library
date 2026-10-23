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

#[cfg(kani)]
mod verification {
    use super::*;

    #[kani::proof]
    #[kani::unwind(5)]
    fn proof_square_grid() {
        let cells: [[bool; 4]; 3] = kani::any();
        let mut square_area = 0;
        for top in 0..3 {
            for left in 0..4 {
                for size in 1..=(3 - top).min(4 - left) {
                    if (top..top + size).all(|i| (left..left + size).all(|j| cells[i][j])) {
                        square_area = square_area.max(size * size);
                    }
                }
            }
        }
        kani::assert(
            largest_square(3, 4, |i, j| cells[i][j]) == square_area,
            "For every 3-by-4 Boolean grid, square DP returns the maximum area of an all-true square, compared with exhaustive square regions",
        );
        for (h, w) in [(0, 4), (3, 0)] {
            kani::assert(
                largest_square(h, w, |i, j| cells[i][j]) == 0,
                "A zero-height or zero-width grid has maximum square area zero",
            );
        }
        kani::cover!(square_area == 0);
        kani::cover!(square_area == 4);
        kani::cover!(square_area == 9);
    }
}
