use super::{Dyadic, Graph, Neighbor, TopologicalSortExt, UsizeGraph, VertexMap};
use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    hash::Hash,
    marker::PhantomData,
    ops::ControlFlow,
};

pub use graph::*;
pub use impartial::*;
pub use partisan::*;
pub use scoring::*;
pub use value::GameValue;

mod graph;
mod impartial;
mod partisan;
mod scoring;
mod value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Player {
    Left,
    Right,
}

impl Player {
    pub fn opponent(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

/// Outcome for the player whose turn it is. A draw is an infinite play.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameOutcome {
    Win,
    Draw,
    Lose,
}

impl GameOutcome {
    fn from_win(win: bool) -> Self {
        if win { Self::Win } else { Self::Lose }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartisanOutcome {
    Next,
    Previous,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameMismatch<S, V> {
    pub state: S,
    pub expected: V,
    pub actual: V,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluatedMove<S, L, V> {
    pub next: Neighbor<S, L>,
    pub value: V,
}

/// Owns a fixed game definition and reusable analysis caches.
/// Move generators may collect statistics, but must keep the rules unchanged.
/// Closure-based games must be finite and acyclic; detected cycles panic.
/// Outcome queries stop exploring a position once a winning move is found.
/// Use `impartial_graph` for Win/Draw/Lose analysis of cyclic graphs.
///
/// ```
/// use competitive::algorithm::{GameAnalyzer, GameOutcome};
/// let mut nim = GameAnalyzer::new().impartial(|&n: &usize| 0..n);
/// assert_eq!(nim.grundy().eval(&3), 3);
/// assert_eq!(nim.sum().grundy().eval(&[3, 3]), 0);
/// assert_eq!(nim.misere().outcome().eval(&1), GameOutcome::Lose);
/// assert_eq!(nim.outcome().eval(&1), GameOutcome::Win);
/// assert_eq!(nim.grundy().verify(0..10, |&n| n as u64), None);
/// ```
#[derive(Default)]
pub struct GameAnalyzer<G = ()> {
    game: G,
}

impl GameAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct GameAnalysis<'a, G, S, V, const MISERE: bool = false, F = fn(&mut G, &S) -> V>
where
    S: ?Sized,
{
    game: &'a mut G,
    evaluate: F,
    marker: PhantomData<fn(&S) -> V>,
}

impl<'a, G, S, V, const MISERE: bool, F> GameAnalysis<'a, G, S, V, MISERE, F>
where
    S: ?Sized,
    F: FnMut(&mut G, &S) -> V,
{
    pub fn eval(&mut self, state: &S) -> V {
        (self.evaluate)(self.game, state)
    }

    /// Keeps the input items, their order and duplicates; dependency states are omitted.
    pub fn table<T>(&mut self, states: impl IntoIterator<Item = T>) -> Vec<(T, V)>
    where
        T: Borrow<S>,
    {
        states
            .into_iter()
            .map(|state| {
                let value = self.eval(state.borrow());
                (state, value)
            })
            .collect()
    }

    /// Returns the first mismatch in input order, or `None` if the range agrees.
    pub fn verify<T>(
        &mut self,
        states: impl IntoIterator<Item = T>,
        mut expected: impl FnMut(&S) -> V,
    ) -> Option<GameMismatch<T, V>>
    where
        T: Borrow<S>,
        V: PartialEq,
    {
        for state in states {
            let expected = expected(state.borrow());
            let actual = self.eval(state.borrow());
            if actual != expected {
                return Some(GameMismatch {
                    state,
                    expected,
                    actual,
                });
            }
        }
        None
    }

    fn new(game: &'a mut G, evaluate: F) -> Self {
        Self {
            game,
            evaluate,
            marker: PhantomData,
        }
    }
}

pub struct Misere<'a, G> {
    game: &'a mut G,
}

pub struct GameSum<'a, G, const MISERE: bool = false> {
    game: &'a mut G,
}

impl<'a, G> Misere<'a, G> {
    pub fn sum(self) -> GameSum<'a, G, true> {
        GameSum { game: self.game }
    }
}

impl<'a, G> GameSum<'a, G> {
    pub fn misere(self) -> GameSum<'a, G, true> {
        GameSum { game: self.game }
    }
}

fn game_mex(mut values: Vec<u64>) -> u64 {
    for i in 0..values.len() {
        while values[i] < values.len() as u64 {
            let j = values[i] as usize;
            if values[j] == values[i] {
                break;
            }
            values.swap(i, j);
        }
    }
    values
        .iter()
        .enumerate()
        .position(|(i, &g)| i as u64 != g)
        .unwrap_or(values.len()) as u64
}

fn canonical_sum<S>(mut state: Vec<S>, ids: &mut HashMap<S, usize>) -> Vec<S>
where
    S: Clone + Eq + Hash,
{
    for s in &state {
        if !ids.contains_key(s) {
            ids.insert(s.clone(), ids.len());
        }
    }
    state.sort_unstable_by_key(|s| ids[s]);
    state
}

fn memoized_game<S, V, I, A, T>(
    state: &S,
    cache: &mut HashMap<S, V>,
    active: &mut HashSet<S>,
    expand: &mut impl FnMut(&S) -> ControlFlow<V, (I, A)>,
    advance: &mut impl FnMut(&S, &mut I) -> Option<(S, T)>,
    feed: &mut impl FnMut(&mut A, T, V) -> ControlFlow<V>,
    finish: &mut impl FnMut(&S, A) -> V,
) -> V
where
    S: Clone + Eq + Hash,
    V: Clone,
{
    if let Some(value) = cache.get(state) {
        return value.clone();
    }
    assert!(active.insert(state.clone()), "cycle in a finite game");
    let value = match expand(state) {
        ControlFlow::Break(value) => value,
        ControlFlow::Continue((mut moves, mut accumulator)) => 'search: {
            while let Some((next, edge)) = advance(state, &mut moves) {
                let child = memoized_game(&next, cache, active, expand, advance, feed, finish);
                if let ControlFlow::Break(value) = feed(&mut accumulator, edge, child) {
                    break 'search value;
                }
            }
            finish(state, accumulator)
        }
    };
    active.remove(state);
    cache.insert(state.clone(), value.clone());
    value
}

fn memoized_wins<S, I>(
    state: &S,
    cache: &mut HashMap<S, bool>,
    mut expand: impl FnMut(&S) -> ControlFlow<bool, I>,
    mut advance: impl FnMut(&S, &mut I) -> Option<S>,
    misere: bool,
) -> bool
where
    S: Clone + Eq + Hash,
{
    memoized_game(
        state,
        cache,
        &mut HashSet::new(),
        &mut |s| expand(s).map_continue(|moves| (moves, false)),
        &mut |s, moves| advance(s, moves).map(|next| (next, ())),
        &mut |has_moves, (), child| {
            *has_moves = true;
            if child {
                ControlFlow::Continue(())
            } else {
                ControlFlow::Break(true)
            }
        },
        &mut |_, has_moves| !has_moves && misere,
    )
}
