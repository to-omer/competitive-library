use super::{
    Convolve998244353, ConvolveSteps, MInt, MIntBase, One, Zero,
    montgomery::{MInt998244353, Modulo998244353},
};

pub fn number_of_increasing_sequences_between<M, C>(a: &[usize], b: &[usize]) -> MInt<M>
where
    M: MIntBase,
    C: ConvolveSteps<T = Vec<MInt<M>>>,
{
    let n = a.len();
    assert_eq!(n, b.len());
    if n == 0 {
        return MInt::<M>::one();
    }
    let mut a = a.to_vec();
    let mut b = b.to_vec();
    for i in 1..n {
        if a[i - 1] > a[i] {
            a[i] = a[i - 1];
        }
    }
    for i in (1..n).rev() {
        if b[i - 1] > b[i] {
            b[i - 1] = b[i];
        }
    }
    if a.iter().zip(b.iter()).any(|(&l, &r)| l >= r) {
        return MInt::<M>::zero();
    }
    let len = n + b[n - 1];
    let mut l = vec![0usize; len];
    let mut r = vec![!0usize; len];
    l[len - 1] = b[n - 1] - 1;
    for (i, (&a, &b)) in a.iter().zip(b.iter()).enumerate() {
        l[i + a] = a;
        r[i + b] = b;
    }
    calc::<M, C>(&l, &r, &[MInt::one()], &[MInt::one(), MInt::one()])
        .iter()
        .fold(MInt::zero(), |s, &x| s + x)
}

pub fn number_of_increasing_sequences_between_998244353(a: &[usize], b: &[usize]) -> MInt998244353 {
    number_of_increasing_sequences_between::<Modulo998244353, Convolve998244353>(a, b)
}

struct Shifted<M>
where
    M: MIntBase,
{
    offset: usize,
    data: Vec<MInt<M>>,
}

impl<M> Clone for Shifted<M>
where
    M: MIntBase,
{
    fn clone(&self) -> Self {
        Self {
            offset: self.offset,
            data: self.data.clone(),
        }
    }
}

impl<M> Default for Shifted<M>
where
    M: MIntBase,
{
    fn default() -> Self {
        Self {
            offset: 0,
            data: Vec::new(),
        }
    }
}

impl<M> Shifted<M>
where
    M: MIntBase,
{
    fn new(offset: usize, data: Vec<MInt<M>>) -> Self {
        Self { offset, data }
    }

    fn add(&self, other: &Self) -> Self {
        let offset = self.offset.min(other.offset);
        let tail = (self.offset + self.data.len()).max(other.offset + other.data.len());
        let mut res = vec![MInt::<M>::zero(); tail - offset];
        let self_start = self.offset - offset;
        for (i, &v) in self.data.iter().enumerate() {
            res[self_start + i] += v;
        }
        let other_start = other.offset - offset;
        for (i, &v) in other.data.iter().enumerate() {
            res[other_start + i] += v;
        }
        Self::new(offset, res)
    }

    fn mul<C>(&self, other: &Self) -> Self
    where
        C: ConvolveSteps<T = Vec<MInt<M>>>,
    {
        let c = C::convolve(self.data.clone(), other.data.clone());
        Self::new(self.offset + other.offset, c)
    }

    fn truncate(&self, l: usize, r: usize) -> Self {
        let mut offset = self.offset;
        let mut data = self.data.clone();
        if offset < l {
            let drop = l - offset;
            if drop >= data.len() {
                data.clear();
                return Self::new(l, data);
            }
            data.drain(..drop);
            offset = l;
        }
        if offset < r {
            let len = r - offset;
            if data.len() > len {
                data.truncate(len);
            }
        } else {
            data.clear();
        }
        Self::new(offset, data)
    }

    fn step<C>(
        self,
        s: usize,
        k: usize,
        a: &[usize],
        b: &[usize],
        pow: &[Self],
        g_len: usize,
    ) -> Self
    where
        C: ConvolveSteps<T = Vec<MInt<M>>>,
    {
        if self.data.is_empty() {
            return self;
        }
        if k == 0 {
            let res = self.mul::<C>(&pow[0]);
            return res.truncate(a[s + 1], b[s + 1]);
        }
        let len = 1usize << k;
        let t = s + len;
        let mut l = a[t];
        let mut r = b[t];
        if l < self.offset {
            l = self.offset;
        }
        let max_r = self.offset + self.data.len();
        let shift = (g_len - 1) << k;
        r = r.saturating_sub(shift).min(max_r);
        if l < r {
            let mut res = Shifted::default();
            if l > 0 {
                let f = self.truncate(0, l);
                let f = f.step::<C>(s, k - 1, a, b, pow, g_len);
                res = f.step::<C>(s + (1usize << (k - 1)), k - 1, a, b, pow, g_len);
            }
            let f = self.truncate(l, r);
            let g = f.mul::<C>(&pow[k]);
            res = res.add(&g);
            if r < max_r {
                let f = self.truncate(r, !0usize);
                let f = f.step::<C>(s, k - 1, a, b, pow, g_len);
                let tail = f.step::<C>(s + (1usize << (k - 1)), k - 1, a, b, pow, g_len);
                res = res.add(&tail);
            }
            res
        } else {
            let next = self.step::<C>(s, k - 1, a, b, pow, g_len);
            next.step::<C>(s + (1usize << (k - 1)), k - 1, a, b, pow, g_len)
        }
    }
}

fn calc<M, C>(a: &[usize], b: &[usize], f: &[MInt<M>], g: &[MInt<M>]) -> Vec<MInt<M>>
where
    M: MIntBase,
    C: ConvolveSteps<T = Vec<MInt<M>>>,
{
    if g.is_empty() {
        return vec![];
    }
    let g_len = g.len();
    let mut a = a.to_vec();
    let mut b = b.to_vec();
    let n = a.len();
    for i in 1..n {
        if a[i] < a[i - 1] {
            a[i] = a[i - 1];
        }
        let limit = b[i - 1].saturating_add(g_len - 1);
        if b[i] > limit {
            b[i] = limit;
        }
    }
    for i in (1..n).rev() {
        let limit = a[i].saturating_sub(g_len - 1);
        if a[i - 1] < limit {
            a[i - 1] = limit;
        }
        if b[i - 1] > b[i] {
            b[i - 1] = b[i];
        }
    }
    if a.iter().zip(b.iter()).any(|(&l, &r)| l >= r) {
        return vec![];
    }
    let k = a.len().next_power_of_two().trailing_zeros() as usize;
    let mut pow: Vec<Shifted<M>> = Vec::with_capacity(k + 1);
    pow.push(Shifted::new(0, g.to_vec()));
    for i in 1..k {
        let prev = pow[i - 1].clone();
        pow.push(prev.mul::<C>(&prev));
    }
    let mut pos = 0usize;
    let mut state = Shifted::new(0, f.to_vec());
    state = state.truncate(a[0], b[0]);
    for i in (0..k).rev() {
        let step_len = 1usize << i;
        if pos + step_len < a.len() {
            state = state.step::<C>(pos, i, &a, &b, &pow, g_len);
            pos += step_len;
        }
    }
    let mut res = vec![MInt::<M>::zero(); state.offset];
    res.extend_from_slice(&state.data);
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{rand, tools::Xorshift};
    use std::collections::HashMap;

    fn naive(a: &[usize], b: &[usize]) -> MInt998244353 {
        let mut dp = HashMap::new();
        dp.insert(0, MInt998244353::one());
        for (&l, &r) in a.iter().zip(b.iter()) {
            let mut next = HashMap::new();
            for (&x, &count) in dp.iter() {
                for y in l.max(x)..r {
                    *next.entry(y).or_default() += count;
                }
            }
            dp = next;
        }
        dp.values().sum()
    }

    #[test]
    fn test_number_of_increasing_sequences_between_998244353() {
        let mut rng = Xorshift::default();
        for _ in 0..300 {
            let n = rng.random(1usize..300);
            let max = rng.random(1usize..300);
            rand!(rng, mut a: [0usize..=max; n], mut b: [0usize..=max; n]);
            for i in 0..n {
                if a[i] > b[i] {
                    std::mem::swap(&mut a[i], &mut b[i]);
                }
            }
            let result = number_of_increasing_sequences_between_998244353(&a, &b);
            let expected = naive(&a, &b);
            assert_eq!(result, expected);
        }
    }
}
