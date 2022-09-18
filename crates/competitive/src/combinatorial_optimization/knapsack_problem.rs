use std::{
    cmp::Ordering,
    collections::VecDeque,
    mem::swap,
    ops::{Add, Neg},
};

#[derive(Debug, Clone)]
pub struct KnapsackPloblemSmallWeight {
    pub dp: Vec<i64>,
}

impl KnapsackPloblemSmallWeight {
    pub fn new(max_weight: usize) -> Self {
        let mut dp = vec![std::i64::MIN; max_weight + 1];
        dp[0] = 0;
        Self { dp }
    }
    pub fn max_weight(&self) -> usize {
        self.dp.len() - 1
    }
    pub fn insert(&mut self, value: i64, weight: usize) {
        for i in weight..self.dp.len() {
            if self.dp[i - weight] != std::i64::MIN {
                self.dp[i] = self.dp[i].max(self.dp[i - weight] + value);
            }
        }
    }
    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (i64, usize)>,
    {
        for (value, weight) in iter.into_iter() {
            self.insert(value, weight);
        }
    }
    pub fn insert01(&mut self, value: i64, weight: usize) {
        for i in (weight..self.dp.len()).rev() {
            if self.dp[i - weight] != std::i64::MIN {
                self.dp[i] = self.dp[i].max(self.dp[i - weight] + value);
            }
        }
    }
    pub fn extend01<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (i64, usize)>,
    {
        for (value, weight) in iter.into_iter() {
            self.insert01(value, weight);
        }
    }
    pub fn insert_limitation(&mut self, value: i64, weight: usize, count: usize) {
        for i in 0..weight {
            let mut deq = VecDeque::new();
            let mut j = 0;
            while j * weight + i < self.dp.len() {
                if self.dp[j * weight + i] != std::i64::MIN {
                    let v = self.dp[j * weight + i] - j as i64 * value;
                    while deq.back().map(|&(_, x)| x <= v).unwrap_or_default() {
                        deq.pop_back();
                    }
                    deq.push_back((j, v));
                }
                if let Some((l, v)) = deq.front() {
                    self.dp[j * weight + i] = v + j as i64 * value;
                    if l + count == j {
                        deq.pop_front();
                    }
                }
                j += 1;
            }
        }
    }
    pub fn extend_limitation<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (i64, usize, usize)>,
    {
        for (value, weight, count) in iter.into_iter() {
            self.insert_limitation(value, weight, count);
        }
    }
    pub fn insert_limitation2(&mut self, value: i64, weight: usize, mut count: usize) {
        let mut b = 1;
        while count > 0 {
            let k = b.min(count);
            count -= k;
            for i in (weight * k..self.dp.len()).rev() {
                if self.dp[i - weight * k] != std::i64::MIN {
                    self.dp[i] = self.dp[i].max(self.dp[i - weight * k] + value * k as i64);
                }
            }
            b *= 2;
        }
    }
    pub fn extend_limitation2<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (i64, usize, usize)>,
    {
        for (value, weight, count) in iter.into_iter() {
            self.insert_limitation2(value, weight, count);
        }
    }
    pub fn solve(&self) -> Option<i64> {
        self.dp
            .iter()
            .filter(|&&dp| dp != std::i64::MIN)
            .max()
            .cloned()
    }
    pub fn get(&self, weight: usize) -> Option<i64> {
        if self.dp[weight] != std::i64::MIN {
            Some(self.dp[weight])
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct KnapsackPloblemSmallValue {
    pub dp: Vec<i64>,
}

impl KnapsackPloblemSmallValue {
    pub fn new(max_value: usize) -> Self {
        let mut dp = vec![std::i64::MAX; max_value + 1];
        dp[0] = 0;
        Self { dp }
    }
    pub fn insert(&mut self, value: usize, weight: i64) {
        for i in value..self.dp.len() {
            if self.dp[i - value] != std::i64::MAX {
                self.dp[i] = self.dp[i].min(self.dp[i - value] + weight);
            }
        }
    }
    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (usize, i64)>,
    {
        for (value, weight) in iter.into_iter() {
            self.insert(value, weight);
        }
    }
    pub fn insert01(&mut self, value: usize, weight: i64) {
        for i in (value..self.dp.len()).rev() {
            if self.dp[i - value] != std::i64::MAX {
                self.dp[i] = self.dp[i].min(self.dp[i - value] + weight);
            }
        }
    }
    pub fn extend01<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (usize, i64)>,
    {
        for (value, weight) in iter.into_iter() {
            self.insert01(value, weight);
        }
    }
    pub fn insert_limitation(&mut self, value: usize, weight: i64, mut count: usize) {
        let mut b = 1;
        while count > 0 {
            let k = b.min(count);
            count -= k;
            for i in (value * k..self.dp.len()).rev() {
                if self.dp[i - value * k] != std::i64::MAX {
                    self.dp[i] = self.dp[i].min(self.dp[i - value * k] + weight * k as i64);
                }
            }
            b *= 2;
        }
    }
    pub fn extend_limitation<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (usize, i64, usize)>,
    {
        for (value, weight, count) in iter.into_iter() {
            self.insert_limitation(value, weight, count);
        }
    }
    pub fn solve(&self, max_weight: i64) -> Option<usize> {
        (0..self.dp.len())
            .filter(|&i| self.dp[i] <= max_weight)
            .max()
    }
    pub fn get(&self, value: usize) -> Option<i64> {
        if self.dp[value] != std::i64::MAX {
            Some(self.dp[value])
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ZeroOneKnapsackProblemSmallItems {
    a: Vec<(i64, i64)>,
    b: Vec<(i64, i64)>,
}

impl Default for ZeroOneKnapsackProblemSmallItems {
    fn default() -> Self {
        Self {
            a: vec![(0, 0)],
            b: vec![(0, 0)],
        }
    }
}

impl ZeroOneKnapsackProblemSmallItems {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn insert(&mut self, value: i64, weight: i64) {
        let mut a_iter = self.a.iter().cloned();
        let mut b_iter = self.a.iter().map(|&(v, w)| (v + value, w + weight));
        let mut c = Vec::with_capacity(self.a.len() * 2);
        let (mut a_next, mut b_next) = (a_iter.next(), b_iter.next());
        loop {
            match (a_next, b_next) {
                (Some(a), Some(b)) => match a.1.cmp(&b.1) {
                    Ordering::Less => {
                        c.push(a);
                        a_next = a_iter.next();
                    }
                    Ordering::Equal => {
                        c.push((a.0.max(b.0), a.1));
                        a_next = a_iter.next();
                        b_next = b_iter.next();
                    }
                    Ordering::Greater => {
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
            swap(&mut self.a, &mut self.b);
        }
    }
    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (i64, i64)>,
    {
        for (value, weight) in iter.into_iter() {
            self.insert(value, weight);
        }
    }
    pub fn solve(&self, max_weight: i64) -> i64 {
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

#[derive(Debug, Clone)]
pub struct ZeroOneKnapsackPloblemBranchAndBound {
    items: Vec<Item>,
    gap: Item,
}

#[derive(Copy, Clone, Default, Debug)]
struct Item {
    value: i64,
    weight: i64,
}
impl From<(i64, i64)> for Item {
    fn from(vw: (i64, i64)) -> Self {
        Self {
            value: vw.0,
            weight: vw.1,
        }
    }
}
impl Add for Item {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value + rhs.value,
            weight: self.weight + rhs.weight,
        }
    }
}
impl Neg for Item {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self {
            value: -self.value,
            weight: -self.weight,
        }
    }
}
impl ZeroOneKnapsackPloblemBranchAndBound {
    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (i64, i64)>,
    {
        let mut items: Vec<Item> = iter.into_iter().map(From::from).collect();
        let mut gap = Item::default();
        for item in &mut items {
            if item.weight < 0 {
                gap = gap + *item;
                *item = -*item;
            }
        }
        items.sort_by(|i1, i2| {
            (i2.value as i128 * i1.weight as i128, i2.value)
                .cmp(&(i1.value as i128 * i2.weight as i128, i1.value))
        });
        Self { items, gap }
    }
    fn solve_relax(&self, i: usize, mut max_weight: i64) -> Result<i64, f64> {
        let mut ans = 0i64;
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
    fn dfs(&self, i: usize, cur: Item, max_weight: i64, max_value: &mut i64) -> i64 {
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
        let mut ans = 0i64;
        if cur.weight + self.items[i].weight <= max_weight {
            ans = ans.max(self.dfs(i + 1, cur + self.items[i], max_weight, max_value));
        }
        ans.max(self.dfs(i + 1, cur, max_weight, max_value))
    }
    pub fn solve(&self, max_weight: i64) -> i64 {
        self.dfs(
            0,
            Default::default(),
            max_weight - self.gap.weight,
            &mut 0i64,
        ) + self.gap.value
    }
}
