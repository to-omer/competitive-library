use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
};

pub struct MergingUnionFind<T, F: Fn(&mut T, &mut T)> {
    cells: Vec<UfCell<T>>,
    merge: F,
}

#[derive(Clone, Debug)]
pub enum UfCell<T> {
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
impl<T> UfCell<T> {
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
            .map(|i| UfCell::Root(RefCell::new(RootData::new(init(i), 1))))
            .collect();
        Self { cells, merge }
    }
    pub fn find(&mut self, x: usize) -> usize {
        let p = match &self.cells[x] {
            UfCell::Child(p) => *p,
            UfCell::Root(_) => return x,
        };
        let y = self.find(p);
        self.cells[x] = UfCell::Child(y);
        y
    }
    pub fn find_root(&self, mut x: usize) -> Ref<RootData<T>> {
        loop {
            match &self.cells[x] {
                UfCell::Child(p) => x = *p,
                UfCell::Root(cell) => return cell.borrow(),
            }
        }
    }
    pub fn find_root_mut(&self, mut x: usize) -> RefMut<RootData<T>> {
        loop {
            match &self.cells[x] {
                UfCell::Child(p) => x = *p,
                UfCell::Root(cell) => return cell.borrow_mut(),
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
        self.cells[y] = UfCell::Child(x);
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
    pub fn all_group_members(&mut self) -> HashMap<usize, Vec<usize>> {
        let mut groups_map = HashMap::new();
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
