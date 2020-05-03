#[cargo_snippet::snippet]
pub fn slide_minimum<T: Clone + Ord>(v: &Vec<T>, k: usize) -> Vec<usize> {
    let mut deq = std::collections::VecDeque::new();
    let mut res = vec![];
    for i in 0..v.len() {
        while deq.back().map(|&j| v[j] >= v[i]).unwrap_or(false) {
            deq.pop_back();
        }
        deq.push_back(i);
        if i + 1 >= k {
            let f = *deq.front().unwrap();
            res.push(f);
            if f == i + 1 - k {
                deq.pop_front();
            }
        }
    }
    res
}
