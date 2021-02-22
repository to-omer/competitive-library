#[codesnip::entry]
/// return: [(start, length)]
pub fn run_length_encoding<T: PartialEq>(v: &[T]) -> Vec<(usize, usize)> {
    let mut res = Vec::new();
    for (i, a) in v.iter().enumerate() {
        if let Some((start, len)) = res.last_mut() {
            if &v[*start] == a {
                *len += 1;
                continue;
            }
        }
        res.push((i, 1));
    }
    res
}

#[codesnip::entry]
/// $P_0 =$ `init`, $P_{i+1} = f(P_i)$
///
/// Return (f, g) then P = f, g, g, ...
pub fn rho_path<T, F>(init: T, f: F) -> (Vec<T>, Vec<T>)
where
    T: Clone + Eq + std::hash::Hash,
    F: Fn(&T) -> T,
{
    let mut path = vec![init.clone()];
    let mut visited = std::collections::HashMap::new();
    visited.insert(init, 0);
    let loop_start = loop {
        let next_val = f(path.last().unwrap());
        if let Some(&idx) = visited.get(&next_val) {
            break idx;
        }
        let cnt = path.len();
        path.push(next_val.clone());
        visited.insert(next_val, cnt);
    };
    let looped = path.split_off(loop_start);
    (path, looped)
}

#[codesnip::entry]
/// $y = \left\lfloor\frac{n}{x}\right\rfloor$
///
/// segments that have same x or y
pub fn floor_kernel(n: usize) -> Vec<usize> {
    let m = (n as f64).sqrt() as usize;
    let mut res = Vec::with_capacity(m * 2 + 1);
    for i in 1..=m {
        res.push(i);
    }
    if n / m + 1 != m + 1 {
        res.push(m + 1);
    }
    for i in (1..=m).rev() {
        res.push(n / i + 1);
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_floor_kernel() {
        for n in 1..1000 {
            let k = floor_kernel(n);
            let from = k.iter().cloned().zip(k.iter().cloned().skip(1));
            let to = k.iter().cloned().zip(k.iter().cloned().skip(1)).rev();
            for ((a, b), (c, d)) in from.zip(to) {
                assert!(a < b);
                assert!(c < d);
                for x in a..b {
                    for y in c..d {
                        assert!(x * y <= n);
                    }
                }
            }
        }
    }
}
