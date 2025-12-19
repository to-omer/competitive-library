use std::ops::Index;

#[derive(Clone, Debug)]
pub struct SuffixArray {
    sa: Vec<usize>,
}

impl SuffixArray {
    pub fn new<T: Ord>(text: &[T]) -> Self {
        let n = text.len();
        let mut ord: Vec<_> = (0..n).collect();
        ord.sort_unstable_by_key(|&i| &text[i]);
        let mut s = vec![0usize; n + 1];
        let mut upper = 0usize;
        for (k, &i) in ord.iter().enumerate() {
            if k == 0 || text[ord[k - 1]] != text[i] {
                upper += 1;
            }
            s[i] = upper;
        }
        s[n] = 0;
        let sa = sa_is(&s, upper);
        Self { sa }
    }
}

impl Index<usize> for SuffixArray {
    type Output = usize;
    fn index(&self, index: usize) -> &Self::Output {
        &self.sa[index]
    }
}

fn induced_sort(s: &[usize], upper: usize, is_s: &[bool], lms: &[usize]) -> Vec<usize> {
    let n = s.len();
    let mut sa = vec![!0usize; n];

    let mut bucket_size = vec![0usize; upper + 1];
    for &c in s {
        bucket_size[c] += 1;
    }

    let mut bucket_tail = vec![0usize; upper + 1];
    {
        let mut sum = 0usize;
        for i in 0..=upper {
            sum += bucket_size[i];
            bucket_tail[i] = sum;
        }
    }
    for &pos in lms.iter().rev() {
        let c = s[pos];
        bucket_tail[c] -= 1;
        sa[bucket_tail[c]] = pos;
    }

    let mut bucket_head = vec![0usize; upper + 1];
    {
        let mut sum = 0usize;
        for i in 0..=upper {
            bucket_head[i] = sum;
            sum += bucket_size[i];
        }
    }
    for i in 0..n {
        let v = sa[i];
        if v == !0 || v == 0 {
            continue;
        }
        let p = v - 1;
        if !is_s[p] {
            let c = s[p];
            sa[bucket_head[c]] = p;
            bucket_head[c] += 1;
        }
    }

    let mut bucket_tail = vec![0usize; upper + 1];
    {
        let mut sum = 0usize;
        for i in 0..=upper {
            sum += bucket_size[i];
            bucket_tail[i] = sum;
        }
    }
    for i in (0..n).rev() {
        let v = sa[i];
        if v == !0 || v == 0 {
            continue;
        }
        let p = v - 1;
        if is_s[p] {
            let c = s[p];
            bucket_tail[c] -= 1;
            sa[bucket_tail[c]] = p;
        }
    }

    sa
}

fn sa_is(s: &[usize], upper: usize) -> Vec<usize> {
    let n = s.len();
    match n {
        0 => return vec![],
        1 => return vec![0],
        _ => {}
    }

    let mut is_s = vec![false; n];
    is_s[n - 1] = true;
    for i in (0..n - 1).rev() {
        is_s[i] = if s[i] < s[i + 1] {
            true
        } else if s[i] > s[i + 1] {
            false
        } else {
            is_s[i + 1]
        };
    }

    let mut lms_map = vec![!0; n];
    let mut lms = vec![];
    for i in 1..n {
        if is_s[i] && !is_s[i - 1] {
            lms_map[i] = lms.len();
            lms.push(i);
        }
    }

    let mut sa = induced_sort(s, upper, &is_s, &lms);

    if lms.len() > 1 {
        let mut rec_s = vec![0usize; lms.len()];
        let mut rec_upper = 0usize;
        let mut prev = !0usize;
        for &pos in &sa {
            if pos != !0 && lms_map[pos] != !0 {
                if prev != !0 {
                    let mut offset = 0usize;
                    loop {
                        if s[prev + offset] != s[pos + offset]
                            || is_s[prev + offset] != is_s[pos + offset]
                        {
                            rec_upper += 1;
                            break;
                        }
                        if offset != 0 {
                            let prev_is_lms = lms_map[prev + offset] != !0;
                            let pos_is_lms = lms_map[pos + offset] != !0;
                            if prev_is_lms || pos_is_lms {
                                rec_upper += (prev_is_lms != pos_is_lms) as usize;
                                break;
                            }
                        }
                        offset += 1;
                    }
                }
                rec_s[lms_map[pos]] = rec_upper;
                prev = pos;
            }
        }

        let rec_sa = if rec_upper + 1 == lms.len() {
            let mut rec_sa = vec![0usize; lms.len()];
            for i in 0..lms.len() {
                rec_sa[rec_s[i]] = i;
            }
            rec_sa
        } else {
            sa_is(&rec_s, rec_upper)
        };

        let mut ordered_lms = Vec::with_capacity(lms.len());
        for &i in &rec_sa {
            ordered_lms.push(lms[i]);
        }
        sa = induced_sort(s, upper, &is_s, &ordered_lms);
    }

    sa
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    #[test]
    fn test_suffix_array() {
        let mut rng = Xorshift::default();
        for _ in 0..500 {
            let n = rng.random(0..=100);
            let m = rng.random(1..=100);
            let s: Vec<_> = rng.random_iter(1..=m).take(n).collect();
            let sa = SuffixArray::new(&s);
            let mut suffixes: Vec<_> = (0..=n).collect();
            suffixes.sort_unstable_by_key(|&i| &s[i..]);
            assert_eq!(sa.sa, suffixes);
        }
    }
}
