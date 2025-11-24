use std::collections::HashMap;

#[derive(Debug)]
struct State {
    next: HashMap<usize, usize>,
    link: usize,
    len: usize,
}

#[derive(Debug)]
pub struct SuffixAutomaton {
    states: Vec<State>,
    last: usize,
}

impl Default for SuffixAutomaton {
    fn default() -> Self {
        Self {
            states: vec![State {
                next: HashMap::new(),
                link: !0,
                len: 0,
            }],
            last: 0,
        }
    }
}

impl SuffixAutomaton {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn state_size(&self) -> usize {
        self.states.len()
    }

    pub fn transitions(&self, state: usize) -> &HashMap<usize, usize> {
        &self.states[state].next
    }

    pub fn length(&self, state: usize) -> usize {
        self.states[state].len
    }

    pub fn link(&self, state: usize) -> usize {
        self.states[state].link
    }

    pub fn number_of_substrings(&self) -> usize {
        let mut total = 0;
        for state in 1..self.state_size() {
            let link = self.link(state);
            total += self.length(state) - self.length(link);
        }
        total
    }

    fn push(&mut self, c: usize) {
        let new_node = self.states.len();
        let last = self.last;
        self.states.push(State {
            next: HashMap::new(),
            link: !0,
            len: self.states[last].len + 1,
        });
        let mut p = last;
        while p != !0 && !self.states[p].next.contains_key(&c) {
            self.states[p].next.insert(c, new_node);
            p = self.states[p].link;
        }
        let q = if p == !0 { 0 } else { self.states[p].next[&c] };
        if p == !0 || self.states[p].len + 1 == self.states[q].len {
            self.states[new_node].link = q;
        } else {
            let new_q = self.states.len();
            self.states.push(State {
                next: self.states[q].next.clone(),
                link: self.states[q].link,
                len: self.states[p].len + 1,
            });
            self.states[q].link = new_q;
            self.states[new_node].link = new_q;
            while p != !0 && self.states[p].next[&c] == q {
                self.states[p].next.insert(c, new_q);
                p = self.states[p].link;
            }
        }
        self.last = new_node;
    }
}

impl FromIterator<usize> for SuffixAutomaton {
    fn from_iter<I: IntoIterator<Item = usize>>(iter: I) -> Self {
        let mut sa = SuffixAutomaton::new();
        sa.extend(iter);
        sa
    }
}

impl Extend<usize> for SuffixAutomaton {
    fn extend<T: IntoIterator<Item = usize>>(&mut self, iter: T) {
        for c in iter {
            self.push(c);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    #[test]
    fn test_suffix_automaton() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            let csize = rng.random(1usize..=10);
            let n = rng.random(1usize..=100);
            let s: Vec<usize> = rng.random_iter(0usize..csize).take(n).collect();
            let sa = SuffixAutomaton::from_iter(s.iter().cloned());
            let mut len = vec![0; sa.state_size()];

            for i in 0..n {
                let mut state = 0;
                for (j, &c) in s[i..].iter().enumerate() {
                    assert!(sa.transitions(state).contains_key(&c));
                    state = sa.transitions(state)[&c];
                    len[state] = len[state].max(j + 1);
                }
            }
            for (state, &len) in len.iter().enumerate() {
                assert_eq!(sa.length(state), len);
            }
            for state in 1..sa.state_size() {
                assert_ne!(sa.link(state), !0);
            }
        }
    }

    #[test]
    fn test_number_of_substrings() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            let csize = rng.random(1usize..=10);
            let n = rng.random(1usize..=100);
            let s: Vec<usize> = rng.random_iter(0usize..csize).take(n).collect();
            let sa = SuffixAutomaton::from_iter(s.iter().cloned());
            let mut substrings = std::collections::HashSet::new();
            for i in 0..n {
                for j in i + 1..=n {
                    substrings.insert(&s[i..j]);
                }
            }
            assert_eq!(sa.number_of_substrings(), substrings.len());
        }
    }
}
