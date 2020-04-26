use crate::algebra::base::Group;
use cargo_snippet::snippet;

#[snippet("UnionFind")]
#[snippet("WeightedUnionFind")]
use std::collections::HashMap;

#[snippet("UnionFind")]
#[derive(Clone, Debug)]
pub struct UnionFind {
    parents: Vec<isize>,
}
#[snippet("UnionFind")]
impl UnionFind {
    pub fn new(n: usize) -> UnionFind {
        let parents = vec![-1; n];
        UnionFind { parents: parents }
    }

    pub fn find(&mut self, x: usize) -> usize {
        if self.parents[x] < 0 {
            x
        } else {
            let xx = self.parents[x] as usize;
            let y = self.find(xx);
            self.parents[x] = y as isize;
            y
        }
    }

    pub fn unite(&mut self, x: usize, y: usize) -> bool {
        use std::mem::swap;
        let mut x = self.find(x);
        let mut y = self.find(y);
        if x == y {
            return false;
        }
        if self.parents[x] > self.parents[y] {
            swap(&mut x, &mut y);
        }
        self.parents[x] += self.parents[y];
        self.parents[y] = x as isize;
        true
    }

    pub fn size(&mut self, x: usize) -> usize {
        let x = self.find(x);
        (-self.parents[x]) as usize
    }

    pub fn same(&mut self, x: usize, y: usize) -> bool {
        self.find(x) == self.find(y)
    }

    pub fn members(&mut self, x: usize) -> Vec<usize> {
        let root = self.find(x);
        (0..self.parents.len())
            .into_iter()
            .filter(|i| self.find(*i) == root)
            .collect::<Vec<usize>>()
    }

    pub fn roots(&mut self) -> Vec<usize> {
        (0..self.parents.len())
            .into_iter()
            .filter(|i| self.parents[*i] < 0)
            .collect::<Vec<usize>>()
    }

    pub fn all_group_members(&mut self) -> HashMap<usize, Vec<usize>> {
        let mut groups_map = HashMap::new();
        for x in 0..self.parents.len() {
            let r = self.find(x);
            groups_map.entry(r).or_insert(vec![]).push(x);
        }
        groups_map
    }
}

#[test]
fn test_union_find() {
    let mut uf = UnionFind::new(10);
    uf.unite(0, 1);
    uf.unite(0, 2);
    uf.unite(2, 9);
    uf.unite(3, 5);
    uf.unite(3, 7);
    assert!(uf.same(0, 1));
    assert!(uf.same(0, 2));
    assert!(uf.same(2, 9));
    assert!(uf.same(3, 5));
    assert!(uf.same(3, 7));
    assert!(!uf.same(0, 3));
    assert!(!uf.same(0, 6));
    assert_eq!(uf.size(0), 4);
    assert_eq!(uf.size(1), 4);
    assert_eq!(uf.size(3), 3);
    // println!("{:?}", uf.all_group_members())
}

#[snippet("WeightedUnionFind")]
#[derive(Clone, Debug)]
pub struct WeightedUnionFind<G: Group> {
    group: G,
    parents: Vec<isize>,
    diff: Vec<G::T>,
}
#[snippet("WeightedUnionFind")]
impl<G: Group> WeightedUnionFind<G> {
    pub fn new(n: usize, group: G) -> Self {
        let parents = vec![-1; n];
        let diff = vec![group.unit().clone(); n];
        WeightedUnionFind {
            parents: parents,
            diff: diff,
            group: group,
        }
    }
    pub fn find(&mut self, x: usize) -> usize {
        if self.parents[x] < 0 {
            x
        } else {
            let px = self.parents[x] as usize;
            let y = self.find(px);
            let w = self.group.operate(&self.diff[x], &self.diff[px]);
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
        let mut w = self.group.operate(&w, &wx);
        w = self.group.operate(&w, &self.group.inverse(&wy));
        use std::mem::swap;
        let mut x = self.find(x);
        let mut y = self.find(y);
        if x == y {
            return false;
        }
        if self.parents[x] > self.parents[y] {
            swap(&mut x, &mut y);
            w = self.group.inverse(&w);
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
            Some(
                self.group
                    .operate(&self.diff[y], &self.group.inverse(&self.diff[x])),
            )
        } else {
            None
        }
    }
    pub fn members(&mut self, x: usize) -> Vec<usize> {
        let root = self.find(x);
        (0..self.parents.len())
            .into_iter()
            .filter(|i| self.find(*i) == root)
            .collect::<Vec<usize>>()
    }
    pub fn roots(&mut self) -> Vec<usize> {
        (0..self.parents.len())
            .into_iter()
            .filter(|i| self.parents[*i] < 0)
            .collect::<Vec<usize>>()
    }
    pub fn all_group_members(&mut self) -> std::collections::HashMap<usize, Vec<usize>> {
        let mut groups_map = std::collections::HashMap::new();
        for x in 0..self.parents.len() {
            let r = self.find(x);
            groups_map.entry(r).or_insert(vec![]).push(x);
        }
        groups_map
    }
}
