use super::Monoid;
use std::collections::VecDeque;

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
    pub fn insert_once_at(&mut self, node: usize, ch: usize) -> usize {
        let index = node * self.char_size + ch;
        if self.child[index] == !0 {
            self.child[index] = self.node_size;
            self.child.resize(self.child.len() + self.char_size, !0);
            self.node_size += 1;
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
    pub fn next_node(&self, node: usize, ch: usize) -> Option<usize> {
        let index = node * self.char_size + ch;
        if self.child[index] == !0 {
            None
        } else {
            Some(self.child[index])
        }
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

#[cfg(kani)]
mod verification {
    use super::*;

    fn check_failure_two_binary_branches<const LEFT: usize, const RIGHT: usize>() {
        use crate::algebra::BitOrOperation;

        let left = LEFT;
        let right = RIGHT;
        // Prefixes of the two words [0, left] and [1, right], with fixed node IDs.
        let mut trie = Trie {
            child: vec![
                1,
                2,
                if left == 0 { 3 } else { usize::MAX },
                if left == 1 { 3 } else { usize::MAX },
                if right == 0 { 4 } else { usize::MAX },
                if right == 1 { 4 } else { usize::MAX },
                usize::MAX,
                usize::MAX,
                usize::MAX,
                usize::MAX,
            ],
            node_size: 5,
            char_size: 2,
        };
        let original: [u8; 5] = kani::any();
        let mut labels = original;
        trie.build_failure::<BitOrOperation<u8>>(&mut labels);
        kani::assert(
            labels[0] == original[0]
                && labels[1] == (original[1] | original[0])
                && labels[2] == (original[2] | original[0])
                && labels[3] == (original[3] | original[left + 1] | original[0])
                && labels[4] == (original[4] | original[right + 1] | original[0]),
            "For all four tries formed by binary words 0a and 1b and arbitrary u8 node labels, failure construction aggregates exactly the labels of every stored suffix, including the node and root",
        );
        let words = [[0usize; 2], [0, 0], [1, 0], [0, left], [1, right]];
        let lengths = [0, 1, 1, 2, 2];
        let node = kani::any::<u8>() as usize;
        kani::assume(node < 5);
        let symbol = usize::from(kani::any::<bool>());
        let last = if lengths[node] == 1 {
            words[node][0]
        } else {
            words[node][1]
        };
        let expected = if lengths[node] > 0 && last == 0 && symbol == left {
            3
        } else if lengths[node] > 0 && last == 1 && symbol == right {
            4
        } else {
            symbol + 1
        };
        kani::assert(
            trie.next_node(node, symbol) == Some(expected),
            "Every completed transition in these four binary tries reaches the longest stored prefix that is a suffix after consuming the next symbol",
        );
        kani::cover!(node == 0);
        kani::cover!(node == 4);
    }

    #[kani::proof]
    #[kani::unwind(7)]
    fn proof_failure_binary_00_11() {
        check_failure_two_binary_branches::<0, 1>();
    }

    #[kani::proof]
    #[kani::unwind(7)]
    fn proof_failure_binary_00_10() {
        check_failure_two_binary_branches::<0, 0>();
    }

    #[kani::proof]
    #[kani::unwind(7)]
    fn proof_failure_binary_01_10() {
        check_failure_two_binary_branches::<1, 0>();
    }

    #[kani::proof]
    #[kani::unwind(7)]
    fn proof_failure_binary_01_11() {
        check_failure_two_binary_branches::<1, 1>();
    }

    #[kani::proof]
    #[kani::unwind(7)]
    fn proof_failure_composition_order() {
        use crate::algebra::{Associative, Magma, Unital};

        enum Composition {}
        impl Magma for Composition {
            // Truth tables of all functions from bool to bool.
            type T = (bool, bool);
            fn operate(x: &Self::T, y: &Self::T) -> Self::T {
                (if y.0 { x.1 } else { x.0 }, if y.1 { x.1 } else { x.0 })
            }
        }
        impl Associative for Composition {}
        impl Unital for Composition {
            fn unit() -> Self::T {
                (false, true)
            }
        }

        // Prefixes of 01 and 10. Failure links are 3 -> 2 -> 0 and 4 -> 1 -> 0.
        let mut trie = Trie {
            child: vec![
                1,
                2,
                usize::MAX,
                3,
                4,
                usize::MAX,
                usize::MAX,
                usize::MAX,
                usize::MAX,
                usize::MAX,
            ],
            node_size: 5,
            char_size: 2,
        };
        let original: [(bool, bool); 5] = kani::any();
        let mut labels = original;
        trie.build_failure::<Composition>(&mut labels);
        let input: bool = kani::any();
        let root = if input { original[0].1 } else { original[0].0 };
        let first = if root { original[1].1 } else { original[1].0 };
        let second = if root { original[2].1 } else { original[2].0 };
        let expected = [
            root,
            first,
            second,
            if second { original[3].1 } else { original[3].0 },
            if first { original[4].1 } else { original[4].0 },
        ];
        for node in 0..5 {
            let actual = if input {
                labels[node].1
            } else {
                labels[node].0
            };
            kani::assert(
                actual == expected[node],
                "For the trie of 01 and 10 and every assignment of bool-to-bool functions, failure aggregation composes the node with its suffix ancestors in decreasing suffix-length order; this monoid is noncommutative",
            );
        }
        kani::cover!(original[0] == (false, false) && original[1] == (true, false));
    }

    #[kani::proof]
    #[kani::unwind(7)]
    fn proof_failure_unary_chain() {
        use crate::algebra::BitOrOperation;

        let mut trie = Trie {
            child: vec![1, 2, 3, 4, usize::MAX],
            node_size: 5,
            char_size: 1,
        };
        let original: [u8; 5] = kani::any();
        let mut labels = original;
        trie.build_failure::<BitOrOperation<u8>>(&mut labels);
        let mut expected = 0;
        for node in 0..5 {
            expected |= original[node];
            kani::assert(
                labels[node] == expected,
                "For the unary trie of depths zero through four and arbitrary u8 labels, each aggregated label contains exactly all prefix labels through its depth, including the full chain of proper suffixes",
            );
            kani::assert(
                trie.next_node(node, 0) == Some((node + 1).min(4)),
                "Every unary transition advances to the next depth or stays at the longest stored prefix",
            );
        }
    }

    fn check_failure_root_alphabet<const ALPHABET: usize>() {
        use crate::algebra::BitOrOperation;

        let mut trie = Trie::new(ALPHABET);
        let original: u8 = kani::any();
        let mut labels = [original];
        trie.build_failure::<BitOrOperation<u8>>(&mut labels);
        kani::assert(
            labels[0] == original && trie.node_size() == 1,
            "Failure construction for a root-only trie preserves its label and size for alphabet sizes zero through four",
        );
        for symbol in 0..ALPHABET {
            kani::assert(
                trie.next_node(0, symbol) == Some(0),
                "Every symbol of a root-only trie becomes a self-loop",
            );
        }
    }

    #[kani::proof]
    #[kani::unwind(6)]
    fn proof_failure_root_alphabets() {
        check_failure_root_alphabet::<0>();
        check_failure_root_alphabet::<1>();
        check_failure_root_alphabet::<2>();
        check_failure_root_alphabet::<3>();
        check_failure_root_alphabet::<4>();
    }
}
