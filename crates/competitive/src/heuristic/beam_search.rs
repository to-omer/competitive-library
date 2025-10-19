use std::{cmp::Reverse, collections::HashSet, fmt::Debug, hash::Hash};

pub trait ModifiableState: Debug {
    type Operation: Clone + Debug;
    type Score: Clone + Ord + Debug;
    type Hash: Clone + Eq + Hash + Debug;
    type Cands: Iterator<Item = Self::Operation>;
    fn score(&self) -> Self::Score;
    fn hash(&self) -> Self::Hash;
    fn accept(&self) -> bool;
    fn soft_update(
        &mut self,
        op: Self::Operation,
        _score: Self::Score,
        _hash: Self::Hash,
    ) -> Option<(Self::Score, Self::Hash, bool)> {
        self.update(op.clone());
        let res = (self.score(), self.hash(), self.accept());
        self.revert(op);
        Some(res)
    }
    fn update(&mut self, op: Self::Operation) {
        self.change(op);
    }
    fn revert(&mut self, op: Self::Operation) {
        self.change(op);
    }
    fn change(&mut self, _op: Self::Operation) {}
    fn candidates(&self) -> Self::Cands;
}

#[derive(Debug)]
pub struct Candidate<S>
where
    S: ModifiableState,
{
    parent: usize,
    op: S::Operation,
    score: S::Score,
    hash: S::Hash,
    accept: bool,
}

impl<S> Clone for Candidate<S>
where
    S: ModifiableState,
{
    fn clone(&self) -> Self {
        Self {
            parent: self.parent,
            op: self.op.clone(),
            score: self.score.clone(),
            hash: self.hash.clone(),
            accept: self.accept,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Node<S>
where
    S: ModifiableState,
{
    parent: usize,
    child: usize,
    prev: usize,
    next: usize,
    op: S::Operation,
    score: S::Score,
    hash: S::Hash,
}

impl<S> Node<S>
where
    S: ModifiableState,
{
    pub fn new(state: &S, init_op: S::Operation) -> Self {
        Node {
            parent: !0,
            child: !0,
            prev: !0,
            next: !0,
            op: init_op,
            score: state.score(),
            hash: state.hash(),
        }
    }
}

#[derive(Debug)]
pub struct Tree<S>
where
    S: ModifiableState,
{
    state: S,
    latest: usize,
    nodes: Vec<Node<S>>,
    cur_node: usize,
}

impl<S> Tree<S>
where
    S: ModifiableState,
{
    pub fn new(state: S, init_op: S::Operation) -> Self {
        let node = Node::new(&state, init_op);
        Tree {
            state,
            latest: 0,
            nodes: vec![node],
            cur_node: 0,
        }
    }

    fn add_node(&mut self, op: S::Operation, parent: usize, score: S::Score, hash: S::Hash) {
        let next = self.nodes[parent].child;
        if next != !0 {
            self.nodes[next].prev = self.nodes.len();
        }
        self.nodes[parent].child = self.nodes.len();

        self.nodes.push(Node {
            parent,
            child: !0,
            prev: !0,
            next,
            op,
            score,
            hash,
        });
    }

    fn remove_node(&mut self, mut idx: usize) {
        loop {
            let Node {
                prev, next, parent, ..
            } = self.nodes[idx];
            assert_ne!(parent, !0);
            if prev & next == !0 {
                idx = parent;
                continue;
            }

            if prev != !0 {
                self.nodes[prev].next = next;
            } else {
                self.nodes[parent].child = next;
            }
            if next != !0 {
                self.nodes[next].prev = prev;
            }

            break;
        }
    }

    pub fn operations(&self, mut idx: usize) -> Vec<S::Operation> {
        let mut ret = vec![];
        loop {
            let Node { op, parent, .. } = &self.nodes[idx];
            if *parent == !0 {
                break;
            }
            ret.push(op.clone());
            idx = *parent;
        }
        ret.reverse();
        ret
    }

    fn update(&mut self, cands: &mut Vec<Candidate<S>>, beam_weidth: usize, minimize: bool) {
        if cands.len() > beam_weidth {
            if minimize {
                cands.select_nth_unstable_by_key(beam_weidth, |s| s.score.clone());
            } else {
                cands.select_nth_unstable_by_key(beam_weidth, |s| Reverse(s.score.clone()));
            }
            cands.truncate(beam_weidth);
        }
        let len = self.nodes.len();
        for Candidate {
            parent,
            op,
            score,
            hash,
            ..
        } in cands.drain(..)
        {
            self.add_node(op, parent, score, hash);
        }
        for i in self.latest..len {
            if self.nodes[i].child == !0 {
                self.remove_node(i);
            }
        }
        self.latest = len;
    }

    pub fn dfs(&mut self, cands: &mut Vec<Candidate<S>>, set: &HashSet<S::Hash>, single: bool) {
        let node = &self.nodes[self.cur_node];
        if node.child == !0 {
            assert!(node.score == self.state.score());
            assert!(node.hash == self.state.hash());

            for op in self.state.candidates() {
                if let Some((score, hash, accept)) =
                    self.state
                        .soft_update(op.clone(), node.score.clone(), node.hash.clone())
                {
                    if !set.contains(&hash) {
                        cands.push(Candidate {
                            parent: self.cur_node,
                            op,
                            score,
                            hash,
                            accept,
                        });
                    }
                };
            }
        } else {
            let node = self.cur_node;
            let mut child = self.nodes[node].child;
            let next_single = single & (self.nodes[child].next == !0);

            loop {
                self.cur_node = child;
                self.state.update(self.nodes[child].op.clone());
                self.dfs(cands, set, next_single);

                if !next_single {
                    self.state.revert(self.nodes[child].op.clone());
                }
                child = self.nodes[child].next;
                if child == !0 {
                    break;
                }
            }

            if !next_single {
                self.cur_node = node;
            }
        }
    }

    pub fn take_best(
        &self,
        cands: &[Candidate<S>],
        minimize: bool,
    ) -> Option<(S::Score, Vec<S::Operation>)> {
        let cands = cands.iter().filter(|cand| cand.accept);
        if let Some(Candidate {
            op, parent, score, ..
        }) = if minimize {
            cands.min_by_key(|cand| cand.score.clone())
        } else {
            cands.max_by_key(|cand| cand.score.clone())
        } {
            let mut ret = self.operations(*parent);
            ret.push(op.clone());
            Some((score.clone(), ret))
        } else {
            None
        }
    }
}

pub fn beam_search<S>(
    state: S,
    init_op: S::Operation,
    beam_weidth: usize,
    minimize: bool,
) -> Option<(S::Score, Vec<S::Operation>)>
where
    S: ModifiableState,
{
    let mut tree = Tree::new(state, init_op);
    let mut cands = vec![];
    let mut set = HashSet::<S::Hash>::default();
    loop {
        tree.dfs(&mut cands, &set, true);
        if let Some(res) = tree.take_best(&cands, minimize) {
            return Some(res);
        }
        if cands.is_empty() {
            return None;
        }
        set.extend(cands.iter().map(|cand| cand.hash.clone()));
        tree.update(&mut cands, beam_weidth, minimize);
    }
}
