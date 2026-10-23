use super::Monoid;
use std::collections::VecDeque;
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

#[cfg_attr(verus_keep_ghost, verus_verify)]
// Clone itself has no formal postcondition.
#[cfg_attr(verus_keep_ghost, verifier::allow(autoderive_clone_without_spec))]
#[derive(Debug, Clone)]
pub struct Trie {
    child: Vec<usize>,
    node_size: usize,
    char_size: usize,
}
impl Trie {
    pub fn new(char_size: usize) -> Self {
        Self {
            child: vec![!0; char_size],
            node_size: 1,
            char_size,
        }
    }
    pub fn with_capacity(char_size: usize, capacity: usize) -> Self {
        let mut child = Vec::with_capacity(capacity * char_size);
        child.resize(char_size, !0);
        Self {
            child,
            node_size: 1,
            char_size,
        }
    }
    #[cfg_attr(verus_keep_ghost, verus_verify)]
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires
            old(self)@.1 > 0, old(self)@.2 > 0,
            old(self)@.0.len() == old(self)@.1 as int * old(self)@.2 as int,
            node < old(self)@.1, ch < old(self)@.2,
            old(self)@.0.len() + old(self)@.2 <= usize::MAX,
            forall|i: int| 0 <= i < old(self)@.0.len() ==>
                #[trigger] old(self)@.0[i] == !0usize || old(self)@.0[i] < old(self)@.1,
        ensures
            final(self)@.2 == old(self)@.2,
            final(self)@.1 == old(self)@.1
                + if old(self)@.0[node as int * old(self)@.2 as int + ch] == !0usize { 1int } else { 0int },
            result < final(self)@.1,
            final(self)@.0 == if old(self)@.0[node as int * old(self)@.2 as int + ch] == !0usize {
                old(self)@.0.update(node as int * old(self)@.2 as int + ch, old(self)@.1)
                    + Seq::new(old(self)@.2 as nat, |_: int| !0usize)
            } else { old(self)@.0 },
            result == final(self)@.0[node as int * final(self)@.2 as int + ch],
    ))]
    pub fn insert_once_at(&mut self, node: usize, ch: usize) -> usize {
        #[cfg(verus_keep_ghost)]
        proof_decl! { let ghost before = self@; }
        #[cfg(verus_keep_ghost)]
        proof! {
            assert(node as int * self.char_size as int + ch < self.child.len()) by(nonlinear_arith)
                requires node < self.node_size, ch < self.char_size,
                    self.char_size > 0,
                    self.child.len() == self.node_size as int * self.char_size as int;
            assert(self.node_size < usize::MAX) by(nonlinear_arith)
                requires self.char_size > 0,
                    self.child.len() == self.node_size as int * self.char_size as int,
                    self.child.len() + self.char_size <= usize::MAX;
        }
        let index = node * self.char_size + ch;
        if self.child[index] == !0 {
            self.child[index] = self.node_size;
            self.child.resize(self.child.len() + self.char_size, !0);
            self.node_size += 1;
            #[cfg(verus_keep_ghost)]
            proof! {
                #[verifier::proof_note("For any well-formed trie with a nonempty alphabet and room for one more node, inserting an absent edge creates exactly one fresh node with no outgoing edges and preserves all other existing edges")]
                assert(self.child@ =~= before.0.update(index as int, before.1)
                    + Seq::new(before.2 as nat, |_: int| !0usize));
            }
        }
        #[cfg(verus_keep_ghost)]
        proof! {
            if before.0[index as int] != !0usize {
                #[verifier::proof_note("Reinserting an existing edge reuses its node and leaves the trie unchanged")]
                assert(self@ == before);
            }
        }
        self.child[index]
    }
    pub fn insert_at<I>(&mut self, mut node: usize, iter: I) -> Vec<usize>
    where
        I: IntoIterator<Item = usize>,
    {
        let mut path = Vec::new();
        for ch in iter.into_iter() {
            path.push(node);
            node = self.insert_once_at(node, ch);
        }
        path.push(node);
        path
    }
    pub fn insert<I>(&mut self, iter: I) -> Vec<usize>
    where
        I: IntoIterator<Item = usize>,
    {
        self.insert_at(0, iter)
    }
    pub fn find_at<I>(&self, mut node: usize, iter: I) -> Result<usize, usize>
    where
        I: IntoIterator<Item = usize>,
    {
        for ch in iter.into_iter() {
            if let Some(&nnode) = self.child.get(node * self.char_size + ch)
                && nnode != !0
            {
                node = nnode;
            } else {
                return Err(node);
            }
        }
        Ok(node)
    }
    pub fn find<I>(&self, iter: I) -> Result<usize, usize>
    where
        I: IntoIterator<Item = usize>,
    {
        self.find_at(0, iter)
    }
    #[cfg_attr(verus_keep_ghost, verus_verify)]
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires
            self@.0.len() == self@.1 as int * self@.2 as int,
            node < self@.1, ch < self@.2,
        ensures result == if self@.0[node as int * self@.2 as int + ch] == !0usize {
            None
        } else { Some(self@.0[node as int * self@.2 as int + ch]) },
    ))]
    pub fn next_node(&self, node: usize, ch: usize) -> Option<usize> {
        #[cfg(verus_keep_ghost)]
        proof! {
            assert(node as int * self.char_size as int + ch < self.child.len()) by(nonlinear_arith)
                requires node < self.node_size, ch < self.char_size,
                    self.char_size > 0,
                    self.child.len() == self.node_size as int * self.char_size as int;
        }
        let index = node * self.char_size + ch;
        let result = if self.child[index] == !0 {
            None
        } else {
            Some(self.child[index])
        };
        #[cfg(verus_keep_ghost)]
        proof! {
            #[verifier::proof_note("For every valid node and alphabet index in any correctly sized trie table, next_node returns None exactly for an absent edge and otherwise returns its stored destination")]
            assert(result == if self.child@[index as int] == !0usize {
                None
            } else { Some(self.child@[index as int]) });
        }
        result
    }
    pub fn node_size(&self) -> usize {
        self.node_size
    }
    pub fn edges(&self) -> Vec<(usize, usize)> {
        let mut edges = Vec::with_capacity(self.node_size - 1);
        let mut stack = vec![0usize];
        while let Some(node) = stack.pop() {
            for ch in (0..self.char_size).rev() {
                if let Some(nnode) = self.next_node(node, ch) {
                    edges.push((node, nnode));
                    stack.push(nnode);
                }
            }
        }
        edges
    }
    pub fn build_failure<M>(&mut self, dp: &mut [M::T])
    where
        M: Monoid,
    {
        let mut fail = vec![0usize; self.node_size];
        let mut deq = VecDeque::new();
        for ch in 0..self.char_size {
            if self.child[ch] != !0 {
                deq.push_back(self.child[ch]);
            } else {
                self.child[ch] = 0;
            }
        }
        while let Some(node) = deq.pop_front() {
            let f = fail[node];
            dp[node] = M::operate(&dp[node], &dp[f]);
            let base = node * self.char_size;
            let fbase = f * self.char_size;
            for ch in 0..self.char_size {
                let index = base + ch;
                let nnode = self.child[index];
                if nnode != !0 {
                    deq.push_back(nnode);
                    fail[nnode] = self.child[fbase + ch];
                } else {
                    self.child[index] = self.child[fbase + ch];
                }
            }
        }
    }
}

#[cfg(verus_keep_ghost)]
verus! {
    impl View for Trie {
        type V = (Seq<usize>, usize, usize);
        closed spec fn view(&self) -> Self::V {
            (self.child@, self.node_size, self.char_size)
        }
    }
}
