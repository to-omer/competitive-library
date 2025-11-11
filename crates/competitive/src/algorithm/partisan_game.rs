use std::{collections::HashMap, hash::Hash, marker::PhantomData};

type Marker<T> = PhantomData<fn() -> T>;

pub trait PartisanGame {
    type State;
    type LIter: Iterator<Item = Self::State>;
    type RIter: Iterator<Item = Self::State>;
    fn next_left(&mut self, s: &Self::State) -> Self::LIter;
    fn next_right(&mut self, s: &Self::State) -> Self::RIter;
}

pub struct PartisanGamer<S, F, G, L, R>
where
    F: FnMut(&S) -> L,
    L: Iterator<Item = S>,
    G: FnMut(&S) -> R,
    R: Iterator<Item = S>,
{
    f: F,
    g: G,
    _marker: Marker<(S, L, R)>,
}

impl<S, F, G, L, R> PartisanGamer<S, F, G, L, R>
where
    F: FnMut(&S) -> L,
    L: Iterator<Item = S>,
    G: FnMut(&S) -> R,
    R: Iterator<Item = S>,
{
    pub fn new(f: F, g: G) -> Self {
        Self {
            f,
            g,
            _marker: PhantomData,
        }
    }
}

impl<S, F, G, L, R> PartisanGame for PartisanGamer<S, F, G, L, R>
where
    F: FnMut(&S) -> L,
    L: Iterator<Item = S>,
    G: FnMut(&S) -> R,
    R: Iterator<Item = S>,
{
    type State = S;
    type LIter = L;
    type RIter = R;
    fn next_left(&mut self, s: &Self::State) -> Self::LIter {
        (self.f)(s)
    }
    fn next_right(&mut self, s: &Self::State) -> Self::RIter {
        (self.g)(s)
    }
}

#[derive(Debug, Clone)]
pub struct PartisanGameAnalyzer<G>
where
    G: PartisanGame,
    G::State: Eq + Hash,
{
    game: G,
    number: HashMap<G::State, i64>,
}

impl<G> PartisanGameAnalyzer<G>
where
    G: PartisanGame,
    G::State: Eq + Hash + Clone,
{
    pub fn new(game: G) -> Self {
        Self {
            game,
            number: Default::default(),
        }
    }
    pub fn eval(&mut self, s: &G::State) -> i64 {
        if let Some(n) = self.number.get(s).cloned() {
            n
        } else {
            let lmax = self
                .game
                .next_left(s)
                .map(|ns| self.eval(&ns))
                .fold(i64::MIN, Ord::max);
            let rmin = self
                .game
                .next_right(s)
                .map(|ns| self.eval(&ns))
                .fold(i64::MAX, Ord::min);
            let res = Self::simple_number(lmax, rmin);
            self.number.insert(s.clone(), res);
            res
        }
    }
    fn simple_number(lmax: i64, rmin: i64) -> i64 {
        const FIX: u32 = 50;
        assert!(lmax < rmin);
        assert!(lmax + 1 != rmin);
        if lmax < 0 && 0 < rmin {
            0
        } else {
            let c = 63 - ((rmin - 1) ^ lmax).leading_zeros();
            if c <= FIX {
                (rmin - 1) >> c << c
            } else if lmax >= 0 {
                ((lmax >> FIX) + 1) << FIX
            } else {
                (rmin - 1) >> FIX << FIX
            }
        }
    }
}
