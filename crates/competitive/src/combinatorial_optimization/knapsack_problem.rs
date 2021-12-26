#[cfg_attr(nightly, codesnip::entry("KnapsackPloblemSmallWeight"))]
pub struct KnapsackPloblemSmallWeight {
    pub dp: Vec<usize>,
}
#[cfg_attr(nightly, codesnip::entry("KnapsackPloblemSmallWeight"))]
impl KnapsackPloblemSmallWeight {
    pub fn new(max_weight: usize) -> Self {
        Self {
            dp: vec![0; max_weight + 1],
        }
    }
    pub fn max_weight(&self) -> usize {
        self.dp.len() - 1
    }
    pub fn insert(&mut self, value: usize, weight: usize) {
        for i in weight..self.dp.len() {
            self.dp[i] = self.dp[i].max(self.dp[i - weight] + value);
        }
    }
    pub fn extend<I: IntoIterator<Item = (usize, usize)>>(&mut self, iter: I) {
        for (value, weight) in iter.into_iter() {
            self.insert(value, weight);
        }
    }
    pub fn insert01(&mut self, value: usize, weight: usize) {
        for i in (weight..self.dp.len()).rev() {
            self.dp[i] = self.dp[i].max(self.dp[i - weight] + value);
        }
    }
    pub fn extend01<I: IntoIterator<Item = (usize, usize)>>(&mut self, iter: I) {
        for (value, weight) in iter.into_iter() {
            self.insert01(value, weight);
        }
    }
    pub fn insert_limitation(&mut self, value: usize, weight: usize, count: usize) {
        use std::collections::VecDeque;
        for i in 0..weight {
            let mut deq = VecDeque::new();
            let mut j = 0;
            while j * weight + i < self.dp.len() {
                let v = self.dp[j * weight + i] as i64 - (j * value) as i64;
                while deq.back().map(|&(_, x)| x <= v).unwrap_or_default() {
                    deq.pop_back();
                }
                deq.push_back((j, v));
                if let Some((l, v)) = deq.front() {
                    self.dp[j * weight + i] = (v + (j * value) as i64) as usize;
                    if l + count == j {
                        deq.pop_front();
                    }
                }
                j += 1;
            }
        }
    }
    pub fn extend_limitation<I: IntoIterator<Item = (usize, usize, usize)>>(&mut self, iter: I) {
        for (value, weight, count) in iter.into_iter() {
            self.insert_limitation(value, weight, count);
        }
    }
    pub fn insert_limitation2(&mut self, value: usize, weight: usize, mut count: usize) {
        let mut b = 1;
        while count > 0 {
            let k = b.min(count);
            count -= k;
            for i in (weight * k..self.dp.len()).rev() {
                self.dp[i] = self.dp[i].max(self.dp[i - weight * k] + value * k);
            }
            b *= 2;
        }
    }
    pub fn extend_limitation2<I: IntoIterator<Item = (usize, usize, usize)>>(&mut self, iter: I) {
        for (value, weight, count) in iter.into_iter() {
            self.insert_limitation2(value, weight, count);
        }
    }
    pub fn solve(&self) -> usize {
        self.dp.iter().max().cloned().unwrap_or_default()
    }
}

#[cfg_attr(nightly, codesnip::entry("KnapsackPloblemSmallValue"))]
pub struct KnapsackPloblemSmallValue {
    pub dp: Vec<usize>,
}
#[cfg_attr(nightly, codesnip::entry("KnapsackPloblemSmallValue"))]
impl KnapsackPloblemSmallValue {
    pub fn new(max_value: usize) -> Self {
        let mut dp = vec![std::usize::MAX; max_value + 1];
        dp[0] = 0;
        Self { dp }
    }
    pub fn insert(&mut self, value: usize, weight: usize) {
        for i in value..self.dp.len() {
            self.dp[i] = self.dp[i].min(self.dp[i - value].saturating_add(weight));
        }
    }
    pub fn extend<I: IntoIterator<Item = (usize, usize)>>(&mut self, iter: I) {
        for (value, weight) in iter.into_iter() {
            self.insert(value, weight);
        }
    }
    pub fn insert01(&mut self, value: usize, weight: usize) {
        for i in (value..self.dp.len()).rev() {
            self.dp[i] = self.dp[i].min(self.dp[i - value].saturating_add(weight));
        }
    }
    pub fn extend01<I: IntoIterator<Item = (usize, usize)>>(&mut self, iter: I) {
        for (value, weight) in iter.into_iter() {
            self.insert01(value, weight);
        }
    }
    pub fn insert_limitation(&mut self, value: usize, weight: usize, mut count: usize) {
        let mut b = 1;
        while count > 0 {
            let k = b.min(count);
            count -= k;
            for i in (value * k..self.dp.len()).rev() {
                self.dp[i] = self.dp[i].min(self.dp[i - value * k].saturating_add(weight * k));
            }
            b *= 2;
        }
    }
    pub fn extend_limitation<I: IntoIterator<Item = (usize, usize, usize)>>(&mut self, iter: I) {
        for (value, weight, count) in iter.into_iter() {
            self.insert_limitation(value, weight, count);
        }
    }
    pub fn solve(&self, max_weight: usize) -> usize {
        (0..self.dp.len())
            .filter(|&i| self.dp[i] <= max_weight)
            .max()
            .unwrap_or_default()
    }
}

#[cfg_attr(nightly, codesnip::entry("ZeroOneKnapsackProblemSmallItems"))]
#[derive(Debug, Clone)]
pub struct ZeroOneKnapsackProblemSmallItems {
    a: Vec<(u64, u64)>,
    b: Vec<(u64, u64)>,
}
#[cfg_attr(nightly, codesnip::entry("ZeroOneKnapsackProblemSmallItems"))]
impl Default for ZeroOneKnapsackProblemSmallItems {
    fn default() -> Self {
        Self {
            a: vec![(0, 0)],
            b: vec![(0, 0)],
        }
    }
}
#[cfg_attr(nightly, codesnip::entry("ZeroOneKnapsackProblemSmallItems"))]
impl ZeroOneKnapsackProblemSmallItems {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn insert(&mut self, value: u64, weight: u64) {
        let mut a_iter = self.a.iter().cloned();
        let mut b_iter = self.a.iter().map(|&(v, w)| (v + value, w + weight));
        let mut c = Vec::with_capacity(self.a.len() * 2);
        let (mut a_next, mut b_next) = (a_iter.next(), b_iter.next());
        loop {
            match (a_next, b_next) {
                (Some(a), Some(b)) => match a.1.cmp(&b.1) {
                    std::cmp::Ordering::Less => {
                        c.push(a);
                        a_next = a_iter.next();
                    }
                    std::cmp::Ordering::Equal => {
                        c.push((a.0.max(b.0), a.1));
                        a_next = a_iter.next();
                        b_next = b_iter.next();
                    }
                    std::cmp::Ordering::Greater => {
                        c.push(b);
                        b_next = b_iter.next();
                    }
                },
                (None, Some(x)) | (Some(x), None) => {
                    c.push(x);
                    c.extend(a_iter);
                    c.extend(b_iter);
                    break;
                }
                (None, None) => {
                    break;
                }
            }
        }
        self.a = c;
        if self.a.len() > self.b.len() {
            std::mem::swap(&mut self.a, &mut self.b);
        }
    }
    pub fn extend<I: IntoIterator<Item = (u64, u64)>>(&mut self, iter: I) {
        for (value, weight) in iter.into_iter() {
            self.insert(value, weight);
        }
    }
    pub fn solve(&self, max_weight: u64) -> u64 {
        let mut ans = 0;
        let mut max = 0;
        let mut i = 0;
        for a in self.a.iter().rev() {
            while i + 1 < self.b.len() && a.1 + self.b[i + 1].1 <= max_weight {
                i += 1;
                max = max.max(self.b[i].0);
            }
            if a.1 + self.b[i].1 <= max_weight {
                ans = ans.max(a.0 + max);
            }
        }
        ans
    }
}

#[cfg_attr(nightly, codesnip::entry("ZeroOneKnapsackPloblemBranchAndBound"))]
pub struct ZeroOneKnapsackPloblemBranchAndBound {
    items: Vec<zero_one_knapsack_problem_branch_and_bound_impls::Item>,
}
#[cfg_attr(nightly, codesnip::entry("ZeroOneKnapsackPloblemBranchAndBound"))]
mod zero_one_knapsack_problem_branch_and_bound_impls {
    use super::*;
    #[derive(Copy, Clone, Default, Debug)]
    pub struct Item {
        pub value: u64,
        pub weight: u64,
    }
    impl From<(u64, u64)> for Item {
        fn from(vw: (u64, u64)) -> Self {
            Self {
                value: vw.0,
                weight: vw.1,
            }
        }
    }
    impl std::ops::Add for Item {
        type Output = Self;
        fn add(self, rhs: Self) -> Self::Output {
            Self {
                value: self.value + rhs.value,
                weight: self.weight + rhs.weight,
            }
        }
    }
    impl ZeroOneKnapsackPloblemBranchAndBound {
        pub fn new(iter: impl IntoIterator<Item = (u64, u64)>) -> Self {
            let mut items: Vec<Item> = iter.into_iter().map(From::from).collect();
            items.sort_by(|i1, i2| {
                (i2.value as u128 * i1.weight as u128, i2.value)
                    .cmp(&(i1.value as u128 * i2.weight as u128, i1.value))
            });
            Self { items }
        }
        fn solve_relax(&self, i: usize, mut max_weight: u64) -> Result<u64, f64> {
            let mut ans = 0u64;
            for &Item { value, weight } in self.items[i..].iter() {
                if max_weight == 0 {
                    break;
                }
                if weight <= max_weight {
                    max_weight -= weight;
                    ans += value;
                } else {
                    return Err(ans as f64 + max_weight as f64 / weight as f64 * value as f64);
                }
            }
            Ok(ans)
        }
        fn dfs(&self, i: usize, cur: Item, max_weight: u64, max_value: &mut u64) -> u64 {
            if i == self.items.len() {
                *max_value = cur.value.max(*max_value);
                return cur.value;
            }
            match self.solve_relax(i, max_weight - cur.weight) {
                Ok(relax) => {
                    *max_value = (relax + cur.value).max(*max_value);
                    return relax + cur.value;
                }
                Err(relax) => {
                    if *max_value as f64 > (relax + cur.value as f64) {
                        return 0;
                    }
                }
            }
            let mut ans = 0u64;
            if cur.weight + self.items[i].weight <= max_weight {
                ans = ans.max(self.dfs(i + 1, cur + self.items[i], max_weight, max_value));
            }
            ans.max(self.dfs(i + 1, cur, max_weight, max_value))
        }
        pub fn solve(&self, max_weight: u64) -> u64 {
            self.dfs(0, Default::default(), max_weight, &mut 0u64)
        }
    }
}
