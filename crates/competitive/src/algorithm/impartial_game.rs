use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    marker::PhantomData,
};

type Marker<T> = PhantomData<fn() -> T>;

pub trait ImpartialGame {
    type State;
    type Iter: Iterator<Item = Self::State>;
    fn next_state(&mut self, s: &Self::State) -> Self::Iter;
}

pub struct ImpartialGamer<S, F, I>
where
    F: FnMut(&S) -> I,
    I: Iterator<Item = S>,
{
    f: F,
    _marker: Marker<(S, I)>,
}

impl<S, F, I> ImpartialGamer<S, F, I>
where
    F: FnMut(&S) -> I,
    I: Iterator<Item = S>,
{
    pub fn new(f: F) -> Self {
        Self {
            f,
            _marker: PhantomData,
        }
    }
}

impl<S, F, I> ImpartialGame for ImpartialGamer<S, F, I>
where
    F: FnMut(&S) -> I,
    I: Iterator<Item = S>,
{
    type State = S;
    type Iter = I;
    fn next_state(&mut self, s: &Self::State) -> Self::Iter {
        (self.f)(s)
    }
}

#[derive(Debug, Clone)]
pub struct ImpartialGameAnalyzer<G>
where
    G: ImpartialGame,
    G::State: Eq + Hash,
{
    game: G,
    grundy: HashMap<G::State, u64>,
}

impl<G> ImpartialGameAnalyzer<G>
where
    G: ImpartialGame,
    G::State: Eq + Hash + Clone,
{
    pub fn new(game: G) -> Self {
        Self {
            game,
            grundy: Default::default(),
        }
    }
    pub fn eval(&mut self, s: &G::State) -> u64 {
        if let Some(g) = self.grundy.get(s).cloned() {
            g
        } else {
            let next: HashSet<_> = self.game.next_state(s).map(|ns| self.eval(&ns)).collect();
            let mut g = 0u64;
            while next.contains(&g) {
                g += 1;
            }
            self.grundy.insert(s.clone(), g);
            g
        }
    }
}
