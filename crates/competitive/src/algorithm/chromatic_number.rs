/// The smallest number of colors needed to color an undirected graph.
pub fn chromatic_number(n: usize, edges: &[(usize, usize)]) -> usize {
    assert!(n < usize::BITS as usize);
    if n == 0 {
        return 0;
    }
    let mut g = vec![0usize; n];
    for &(u, v) in edges {
        g[u] |= 1 << v;
        g[v] |= 1 << u;
    }
    let deg: Vec<_> = g.iter().map(|g| g.count_ones()).collect();
    let mut best = greedy_coloring(&g, &deg);
    search_coloring(&g, &deg, &mut vec![0; n], (1 << n) - 1, 0, &mut best);
    best
}

fn select_vertex(mut cand: usize, deg: &[u32], sat: &[usize]) -> usize {
    let mut v = cand.trailing_zeros() as usize;
    cand &= cand - 1;
    let mut key = (sat[v].count_ones(), deg[v]);
    while cand != 0 {
        let u = cand.trailing_zeros() as usize;
        cand &= cand - 1;
        let next = (sat[u].count_ones(), deg[u]);
        if next > key {
            key = next;
            v = u;
        }
    }
    v
}

fn greedy_coloring(g: &[usize], deg: &[u32]) -> usize {
    let n = g.len();
    let mut rem = (1usize << n) - 1;
    let mut sat = vec![0usize; n];
    let mut used = 0;
    while rem != 0 {
        let v = select_vertex(rem, deg, &sat);
        let c = (!sat[v]).trailing_zeros() as usize;
        used = used.max(c + 1);
        rem &= !(1 << v);
        let mut next = g[v] & rem;
        while next != 0 {
            let u = next.trailing_zeros() as usize;
            next &= next - 1;
            sat[u] |= 1 << c;
        }
    }
    used
}

fn search_coloring(
    g: &[usize],
    deg: &[u32],
    sat: &mut [usize],
    rem: usize,
    used: usize,
    best: &mut usize,
) {
    if rem == 0 {
        *best = used;
        return;
    }
    if used >= *best {
        return;
    }
    let v = select_vertex(rem, deg, sat);
    let rem = rem & !(1 << v);
    let mut colors = ((1 << used) - 1) & !sat[v];
    if used + 1 < *best {
        colors |= 1 << used;
    }
    while colors != 0 {
        let color = colors & colors.wrapping_neg();
        colors ^= color;
        let mut next = g[v] & rem;
        let mut changed = 0usize;
        while next != 0 {
            let u = next.trailing_zeros() as usize;
            next &= next - 1;
            if sat[u] & color == 0 {
                sat[u] |= color;
                changed |= 1 << u;
            }
        }
        search_coloring(
            g,
            deg,
            sat,
            rem,
            used.max(color.trailing_zeros() as usize + 1),
            best,
        );
        while changed != 0 {
            let u = changed.trailing_zeros() as usize;
            changed &= changed - 1;
            sat[u] ^= color;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    #[test]
    fn test_chromatic_number() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            let n = rng.random(0..=12);
            let mut edges = Vec::new();
            for u in 0..n {
                for v in u + 1..n {
                    if rng.gen_bool(0.5) {
                        edges.push((u, v));
                    }
                }
            }
            let mut g = vec![0usize; n];
            for &(u, v) in &edges {
                g[u] |= 1 << v;
                g[v] |= 1 << u;
            }
            let mut independent = vec![true; 1 << n];
            for s in 1usize..1 << n {
                let v = s.trailing_zeros() as usize;
                independent[s] = independent[s ^ (1 << v)] && g[v] & s == 0;
            }
            let mut dp = vec![n; 1 << n];
            dp[0] = 0;
            for s in 1usize..1 << n {
                let first = 1 << s.trailing_zeros();
                let mut subset = s;
                while subset != 0 {
                    if subset & first != 0 && independent[subset] {
                        dp[s] = dp[s].min(dp[s ^ subset] + 1);
                    }
                    subset = (subset - 1) & s;
                }
            }
            assert_eq!(dp[(1 << n) - 1], chromatic_number(n, &edges));
        }
    }
}
