use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct UnionFind {
    parents: Vec<isize>,
}
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
