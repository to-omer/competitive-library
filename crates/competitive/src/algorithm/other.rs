#[snippet::entry]
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

#[snippet::entry]
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
