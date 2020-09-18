use crate::algebra::Group;

#[cargo_snippet::snippet("UnionFind")]
#[derive(Clone, Debug)]
pub struct UnionFind {
    parents: Vec<isize>,
}
#[cargo_snippet::snippet("UnionFind")]
impl UnionFind {
    pub fn new(n: usize) -> Self {
        let parents = vec![-1; n];
        Self { parents }
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
            .filter(|i| self.find(*i) == root)
            .collect::<Vec<usize>>()
    }

    pub fn roots(&mut self) -> Vec<usize> {
        (0..self.parents.len())
            .filter(|i| self.parents[*i] < 0)
            .collect::<Vec<usize>>()
    }

    pub fn all_group_members(&mut self) -> std::collections::HashMap<usize, Vec<usize>> {
        let mut groups_map = std::collections::HashMap::new();
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

#[cargo_snippet::snippet("WeightedUnionFind")]
#[derive(Clone, Debug)]
pub struct WeightedUnionFind<G: Group> {
    group: G,
    parents: Vec<isize>,
    diff: Vec<G::T>,
}
#[cargo_snippet::snippet("WeightedUnionFind")]
impl<G: Group> WeightedUnionFind<G> {
    pub fn new(n: usize, group: G) -> Self {
        let parents = vec![-1; n];
        let diff = vec![group.unit(); n];
        Self {
            parents,
            diff,
            group,
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
        w = self.group.rinv_operate(&w, &wy);
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
            Some(self.group.rinv_operate(&self.diff[y], &self.diff[x]))
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
    pub fn all_group_members(&mut self) -> std::collections::HashMap<usize, Vec<usize>> {
        let mut groups_map = std::collections::HashMap::new();
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

#[cargo_snippet::snippet("MergingUnionFind")]
pub struct MergingUnionFind<T, F: Fn(&mut T, &mut T)> {
    cells: Vec<merging_union_find_impls::UFCell<T>>,
    merge: F,
}
#[cargo_snippet::snippet("MergingUnionFind")]
mod merging_union_find_impls {
    use super::*;
    use std::cell::{Ref, RefCell, RefMut};
    use UFCell::*;
    #[derive(Clone, Debug)]
    pub enum UFCell<T> {
        Child(usize),
        Root(RefCell<RootData<T>>),
    }
    #[derive(Clone, Debug)]
    pub struct RootData<T> {
        pub data: T,
        pub size: usize,
    }
    impl<T> RootData<T> {
        pub fn new(data: T, size: usize) -> Self {
            Self { data, size }
        }
    }
    impl<T> UFCell<T> {
        pub fn is_root(&self) -> bool {
            match self {
                Self::Child(_) => false,
                Self::Root(_) => true,
            }
        }
    }
    impl<T, F: Fn(&mut T, &mut T)> MergingUnionFind<T, F> {
        pub fn new<I: Fn(usize) -> T>(n: usize, init: I, merge: F) -> Self {
            let cells: Vec<_> = (0..n)
                .map(|i| Root(RefCell::new(RootData::new(init(i), 1))))
                .collect();
            Self { cells, merge }
        }
        pub fn find(&mut self, x: usize) -> usize {
            let p = match &self.cells[x] {
                Child(p) => *p,
                Root(_) => return x,
            };
            let y = self.find(p);
            self.cells[x] = Child(y);
            y
        }
        pub fn find_root(&self, mut x: usize) -> Ref<RootData<T>> {
            loop {
                match &self.cells[x] {
                    Child(p) => x = *p,
                    Root(cell) => return cell.borrow(),
                }
            }
        }
        pub fn find_root_mut(&self, mut x: usize) -> RefMut<RootData<T>> {
            loop {
                match &self.cells[x] {
                    Child(p) => x = *p,
                    Root(cell) => return cell.borrow_mut(),
                }
            }
        }
        pub fn unite(&mut self, x: usize, y: usize) -> bool {
            let mut x = self.find(x);
            let mut y = self.find(y);
            if x == y {
                return false;
            }
            if self.size(x) < self.size(y) {
                std::mem::swap(&mut x, &mut y);
            }
            {
                let mut cx = self.find_root_mut(x);
                let mut cy = self.find_root_mut(y);
                (self.merge)(&mut cx.data, &mut cy.data);
                cx.size += cy.size;
            }
            self.cells[y] = Child(x);
            true
        }
        pub fn size(&mut self, x: usize) -> usize {
            self.find_root(x).size
        }
        pub fn same(&mut self, x: usize, y: usize) -> bool {
            self.find(x) == self.find(y)
        }
        pub fn members(&mut self, x: usize) -> Vec<usize> {
            let root = self.find(x);
            (0..self.cells.len())
                .filter(|i| self.find(*i) == root)
                .collect::<Vec<usize>>()
        }
        pub fn roots(&mut self) -> Vec<usize> {
            (0..self.cells.len())
                .filter(|&i| self.cells[i].is_root())
                .collect::<Vec<usize>>()
        }
        pub fn all_group_members(&mut self) -> std::collections::HashMap<usize, Vec<usize>> {
            let mut groups_map = std::collections::HashMap::new();
            for x in 0..self.cells.len() {
                let r = self.find(x);
                groups_map
                    .entry(r)
                    .or_insert_with(|| Vec::with_capacity(self.size(r)))
                    .push(x);
            }
            groups_map
        }
    }
}
