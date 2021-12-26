use crate::graph::UndirectedSparseGraph;

#[cfg_attr(nightly, codesnip::entry("tree_dp", include("SparseGraph")))]
impl UndirectedSparseGraph {
    pub fn tree_dp_bottom_up<T, F>(&self, root: usize, dp: &mut [T], mut f: F)
    where
        F: FnMut(&mut T, &T),
    {
        fn dfs<T, F>(g: &UndirectedSparseGraph, u: usize, p: usize, dp: &mut [T], f: &mut F)
        where
            F: FnMut(&mut T, &T),
        {
            for a in g.adjacencies(u) {
                if a.to != p {
                    dfs(g, a.to, u, dp, f);
                    assert_ne!(u, a.to);
                    let ptr = dp.as_mut_ptr();
                    unsafe { f(&mut *ptr.add(u), &*ptr.add(a.to)) };
                }
            }
        }
        dfs(self, root, !0, dp, &mut f);
    }
    pub fn tree_dp_top_down<T, F>(&self, root: usize, dp: &mut [T], mut f: F)
    where
        F: FnMut(&mut T, &T),
    {
        fn dfs<T, F>(g: &UndirectedSparseGraph, u: usize, p: usize, dp: &mut [T], f: &mut F)
        where
            F: FnMut(&mut T, &T),
        {
            for a in g.adjacencies(u) {
                if a.to != p {
                    assert_ne!(u, a.to);
                    let ptr = dp.as_mut_ptr();
                    unsafe { f(&mut *ptr.add(a.to), &*ptr.add(u)) };
                    dfs(g, a.to, u, dp, f);
                }
            }
        }
        dfs(self, root, !0, dp, &mut f);
    }
}
