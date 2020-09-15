#[cargo_snippet::snippet]
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
