use super::Group;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct WeightedUnionFind<G: Group> {
    parents: Vec<isize>,
    diff: Vec<G::T>,
}
impl<G: Group> WeightedUnionFind<G> {
    pub fn new(n: usize) -> Self {
        let parents = vec![-1; n];
        let diff = vec![G::unit(); n];
        Self { parents, diff }
    }
    pub fn find(&mut self, x: usize) -> usize {
        if self.parents[x] < 0 {
            x
        } else {
            let px = self.parents[x] as usize;
            let y = self.find(px);
            let w = G::operate(&self.diff[x], &self.diff[px]);
            self.diff[x] = w;
            self.parents[x] = y as isize;
            y
        }
    }
    pub fn get_weight(&mut self, x: usize) -> G::T {
        self.find(x);
        self.diff[x].clone()
    }
    pub fn unite(&mut self, x: usize, y: usize, w: G::T) -> bool {
        let wx = self.get_weight(x);
        let wy = self.get_weight(y);
        let mut w = G::operate(&w, &wx);
        w = G::rinv_operate(&w, &wy);
        use std::mem::swap;
        let mut x = self.find(x);
        let mut y = self.find(y);
        if x == y {
            return false;
        }
        if self.parents[x] > self.parents[y] {
            swap(&mut x, &mut y);
            w = G::inverse(&w);
        }
        self.parents[x] += self.parents[y];
        self.parents[y] = x as isize;
        self.diff[y] = w;
        true
    }
    pub fn size(&mut self, x: usize) -> usize {
        let x = self.find(x);
        (-self.parents[x]) as usize
    }
    pub fn is_same(&mut self, x: usize, y: usize) -> bool {
        self.find(x) == self.find(y)
    }
    pub fn get_difference(&mut self, x: usize, y: usize) -> Option<G::T> {
        if self.is_same(x, y) {
            Some(G::rinv_operate(&self.diff[y], &self.diff[x]))
        } else {
            None
        }
    }
    pub fn members(&mut self, x: usize) -> Vec<usize> {
        let root = self.find(x);
        (0..self.parents.len())
            .filter(|i| self.find(*i) == root)
            .collect::<Vec<usize>>()
    }
    pub fn roots(&mut self) -> Vec<usize> {
        (0..self.parents.len())
            .filter(|i| self.parents[*i] < 0)
            .collect::<Vec<usize>>()
    }
    pub fn all_group_members(&mut self) -> HashMap<usize, Vec<usize>> {
        let mut groups_map = HashMap::new();
        for x in 0..self.parents.len() {
            let r = self.find(x);
            groups_map
                .entry(r)
                .or_insert_with(|| Vec::with_capacity(self.size(r)))
                .push(x);
        }
        groups_map
    }
}
