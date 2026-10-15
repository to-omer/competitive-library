use super::{
    ControlFlow, EvaluatedMove, GameAnalysis, GameAnalyzer, GameOutcome, GameSum, GameValue, Hash,
    HashMap, HashSet, Misere, Neighbor, canonical_sum, game_mex, memoized_game, memoized_wins,
};

pub struct Impartial<S, F> {
    moves: F,
    grundy: HashMap<S, u64>,
    outcomes: [HashMap<S, bool>; 2],
    sums: HashMap<Vec<S>, bool>,
    sum_ids: HashMap<S, usize>,
}

impl GameAnalyzer {
    #[allow(clippy::type_complexity)] // The adapter preserves the caller's iterator.
    pub fn impartial<S, F, I>(
        self,
        mut moves: F,
    ) -> GameAnalyzer<
        Impartial<S, impl FnMut(&S) -> std::iter::Map<I::IntoIter, fn(S) -> Neighbor<S, ()>>>,
    >
    where
        F: FnMut(&S) -> I,
        I: IntoIterator<Item = S>,
    {
        let label: fn(S) -> Neighbor<S, ()> = |to| Neighbor::new(to, ());
        self.impartial_moves(move |s: &S| moves(s).into_iter().map(label))
    }

    pub fn impartial_moves<S, L, F, I>(self, moves: F) -> GameAnalyzer<Impartial<S, F>>
    where
        F: FnMut(&S) -> I,
        I: IntoIterator<Item = Neighbor<S, L>>,
    {
        GameAnalyzer {
            game: Impartial {
                moves,
                grundy: HashMap::new(),
                outcomes: std::array::from_fn(|_| HashMap::new()),
                sums: HashMap::new(),
                sum_ids: HashMap::new(),
            },
        }
    }
}

impl<S, F> GameAnalyzer<Impartial<S, F>> {
    pub fn misere(&mut self) -> Misere<'_, Impartial<S, F>> {
        Misere {
            game: &mut self.game,
        }
    }

    pub fn sum(&mut self) -> GameSum<'_, Impartial<S, F>> {
        GameSum {
            game: &mut self.game,
        }
    }

    pub fn clear_cache(&mut self) {
        self.game.grundy.clear();
        self.game.outcomes.iter_mut().for_each(HashMap::clear);
        self.game.sums.clear();
        self.game.sum_ids.clear();
    }
}

impl<S, L, F, I> GameAnalyzer<Impartial<S, F>>
where
    S: Clone + Eq + Hash,
    F: FnMut(&S) -> I,
    I: IntoIterator<Item = Neighbor<S, L>>,
{
    pub fn grundy(&mut self) -> GameAnalysis<'_, Impartial<S, F>, S, u64> {
        GameAnalysis::new(&mut self.game, Impartial::grundy)
    }

    pub fn value(&mut self) -> GameAnalysis<'_, Impartial<S, F>, S, GameValue> {
        GameAnalysis::new(&mut self.game, |g, s| GameValue::nimber(g.grundy(s)))
    }

    pub fn outcome(&mut self) -> GameAnalysis<'_, Impartial<S, F>, S, GameOutcome> {
        GameAnalysis::new(&mut self.game, |g, s| g.outcome(s, false))
    }

    /// `combine` receives each move's label and the value of its destination.
    #[allow(clippy::type_complexity)] // The query owns the fold and its cache.
    pub fn fold<V, C>(
        &mut self,
        mut combine: C,
    ) -> GameAnalysis<'_, Impartial<S, F>, S, V, false, impl FnMut(&mut Impartial<S, F>, &S) -> V>
    where
        V: Clone,
        C: FnMut(&S, &[(L, V)]) -> V,
    {
        let mut cache = HashMap::new();
        GameAnalysis::new(
            &mut self.game,
            move |game: &mut Impartial<S, F>, state: &S| {
                memoized_game(
                    state,
                    &mut cache,
                    &mut HashSet::new(),
                    &mut |s| {
                        ControlFlow::Continue((
                            (game.moves)(s).into_iter().map(|n| (n.to, n.label)),
                            Vec::new(),
                        ))
                    },
                    &mut |_, moves| moves.next(),
                    &mut |values, label, value| {
                        values.push((label, value));
                        ControlFlow::Continue(())
                    },
                    &mut |s, values| combine(s, &values),
                )
            },
        )
    }
}

impl<'a, S, L, F, I> Misere<'a, Impartial<S, F>>
where
    S: Clone + Eq + Hash,
    F: FnMut(&S) -> I,
    I: IntoIterator<Item = Neighbor<S, L>>,
{
    pub fn outcome(self) -> GameAnalysis<'a, Impartial<S, F>, S, GameOutcome, true> {
        GameAnalysis::new(self.game, |g, s| g.outcome(s, true))
    }
}
impl<'a, S, L, F, I> GameSum<'a, Impartial<S, F>>
where
    S: Clone + Eq + Hash,
    F: FnMut(&S) -> I,
    I: IntoIterator<Item = Neighbor<S, L>>,
{
    pub fn grundy(self) -> GameAnalysis<'a, Impartial<S, F>, [S], u64> {
        GameAnalysis::new(self.game, Impartial::sum_grundy)
    }
}
impl<'a, S, L, F, I, const MISERE: bool> GameSum<'a, Impartial<S, F>, MISERE>
where
    S: Clone + Eq + Hash,
    F: FnMut(&S) -> I,
    I: IntoIterator<Item = Neighbor<S, L>>,
{
    pub fn outcome(self) -> GameAnalysis<'a, Impartial<S, F>, [S], GameOutcome, MISERE> {
        GameAnalysis::new(self.game, |g, s| g.sum_outcome(s, MISERE))
    }
}

impl<S, L, F, I, V, const MISERE: bool, E> GameAnalysis<'_, Impartial<S, F>, S, V, MISERE, E>
where
    F: FnMut(&S) -> I,
    I: IntoIterator<Item = Neighbor<S, L>>,
    E: FnMut(&mut Impartial<S, F>, &S) -> V,
{
    pub fn moves(&mut self, state: &S) -> Vec<EvaluatedMove<S, L, V>> {
        (self.game.moves)(state)
            .into_iter()
            .map(|next| {
                let value = self.eval(&next.to);
                EvaluatedMove { next, value }
            })
            .collect()
    }
}

impl<S, L, F, I> GameAnalysis<'_, Impartial<S, F>, S, u64>
where
    S: Clone + Eq + Hash,
    F: FnMut(&S) -> I,
    I: IntoIterator<Item = Neighbor<S, L>>,
{
    pub fn winning_moves(&mut self, state: &S) -> Vec<Neighbor<S, L>> {
        self.game.winning_moves(state, false).collect()
    }
}
impl<S, L, F, I> GameAnalysis<'_, Impartial<S, F>, S, GameValue>
where
    S: Clone + Eq + Hash,
    F: FnMut(&S) -> I,
    I: IntoIterator<Item = Neighbor<S, L>>,
{
    pub fn winning_moves(&mut self, state: &S) -> Vec<Neighbor<S, L>> {
        self.game.winning_moves(state, false).collect()
    }
}
impl<S, L, F, I, const MISERE: bool> GameAnalysis<'_, Impartial<S, F>, S, GameOutcome, MISERE>
where
    S: Clone + Eq + Hash,
    F: FnMut(&S) -> I,
    I: IntoIterator<Item = Neighbor<S, L>>,
{
    pub fn winning_moves(&mut self, state: &S) -> Vec<Neighbor<S, L>> {
        self.game.winning_moves(state, MISERE).collect()
    }
}

#[allow(clippy::type_complexity)] // Sum states and labels both carry component information.
impl<S, L, F, I, V, const MISERE: bool, E> GameAnalysis<'_, Impartial<S, F>, [S], V, MISERE, E>
where
    S: Clone,
    F: FnMut(&S) -> I,
    I: IntoIterator<Item = Neighbor<S, L>>,
    E: FnMut(&mut Impartial<S, F>, &[S]) -> V,
{
    pub fn moves(&mut self, state: &[S]) -> Vec<EvaluatedMove<Vec<S>, (usize, L), V>> {
        let mut result = Vec::new();
        for (i, component) in state.iter().enumerate() {
            for next in (self.game.moves)(component) {
                let mut to = state.to_vec();
                to[i] = next.to;
                let value = self.eval(&to);
                result.push(EvaluatedMove {
                    next: Neighbor::new(to, (i, next.label)),
                    value,
                });
            }
        }
        result
    }
}
#[allow(clippy::type_complexity)] // Sum states and labels both carry component information.
impl<S, L, F, I> GameAnalysis<'_, Impartial<S, F>, [S], u64>
where
    S: Clone + Eq + Hash,
    F: FnMut(&S) -> I,
    I: IntoIterator<Item = Neighbor<S, L>>,
{
    pub fn winning_moves(&mut self, state: &[S]) -> Vec<Neighbor<Vec<S>, (usize, L)>> {
        self.game.sum_winning_moves(state)
    }
}
#[allow(clippy::type_complexity)] // Sum states and labels both carry component information.
impl<S, L, F, I, const MISERE: bool> GameAnalysis<'_, Impartial<S, F>, [S], GameOutcome, MISERE>
where
    S: Clone + Eq + Hash,
    F: FnMut(&S) -> I,
    I: IntoIterator<Item = Neighbor<S, L>>,
{
    pub fn winning_moves(&mut self, state: &[S]) -> Vec<Neighbor<Vec<S>, (usize, L)>> {
        if let [single] = state {
            return self
                .game
                .winning_moves(single, MISERE)
                .map(|next| Neighbor::new(vec![next.to], (0, next.label)))
                .collect();
        }
        if !MISERE {
            return self.game.sum_winning_moves(state);
        }
        let mut result = Vec::new();
        for (i, component) in state.iter().enumerate() {
            for next in (self.game.moves)(component) {
                let mut to = state.to_vec();
                to[i] = next.to;
                if self.eval(&to) == GameOutcome::Lose {
                    result.push(Neighbor::new(to, (i, next.label)));
                }
            }
        }
        result
    }
}

impl<S, L, F, I> Impartial<S, F>
where
    S: Clone + Eq + Hash,
    F: FnMut(&S) -> I,
    I: IntoIterator<Item = Neighbor<S, L>>,
{
    fn grundy(&mut self, state: &S) -> u64 {
        memoized_game(
            state,
            &mut self.grundy,
            &mut HashSet::new(),
            &mut |s| {
                if self.outcomes[0].get(s) == Some(&false) {
                    return ControlFlow::Break(0);
                }
                let moves = (self.moves)(s).into_iter().map(|n| (n.to, ()));
                ControlFlow::Continue((moves, Vec::new()))
            },
            &mut |_, moves| moves.next(),
            &mut |values, (), value| {
                values.push(value);
                ControlFlow::Continue(())
            },
            &mut |_, values| game_mex(values),
        )
    }

    fn outcome(&mut self, state: &S, misere: bool) -> GameOutcome {
        let win = memoized_wins(
            state,
            &mut self.outcomes[usize::from(misere)],
            |s| {
                if !misere && let Some(&value) = self.grundy.get(s) {
                    return ControlFlow::Break(value != 0);
                }
                let moves = (self.moves)(s).into_iter().map(|n| n.to);
                ControlFlow::Continue(moves)
            },
            |_, moves| moves.next(),
            misere,
        );
        GameOutcome::from_win(win)
    }

    fn winning_moves(&mut self, state: &S, misere: bool) -> impl Iterator<Item = Neighbor<S, L>> {
        (self.moves)(state).into_iter().filter(move |next| {
            if !misere && let Some(&value) = self.grundy.get(&next.to) {
                value == 0
            } else {
                self.outcome(&next.to, misere) == GameOutcome::Lose
            }
        })
    }

    fn sum_grundy(&mut self, state: &[S]) -> u64 {
        match state {
            [] => return 0,
            [single] => return self.grundy(single),
            _ => {}
        }
        let (xor, odd) = self.reduce_sum(state);
        odd.into_iter().fold(xor, |value, s| value ^ self.grundy(s))
    }

    #[allow(clippy::type_complexity)] // Sum states and labels both carry component information.
    fn sum_winning_moves(&mut self, state: &[S]) -> Vec<Neighbor<Vec<S>, (usize, L)>> {
        let xor = self.sum_grundy(state);
        let mut result = Vec::new();
        if xor == 0 {
            return result;
        }
        for (i, component) in state.iter().enumerate() {
            let target = xor ^ self.grundy(component);
            for next in (self.moves)(component) {
                if self.grundy(&next.to) == target {
                    let mut to = state.to_vec();
                    to[i] = next.to;
                    result.push(Neighbor::new(to, (i, next.label)));
                }
            }
        }
        result
    }

    fn sum_outcome(&mut self, state: &[S], misere: bool) -> GameOutcome {
        if state.is_empty() {
            return GameOutcome::from_win(misere);
        }
        if let [single] = state {
            return self.outcome(single, misere);
        }
        let win = if misere {
            if let Some(&win) = self.sums.get(state) {
                win
            } else {
                memoized_wins(
                    &canonical_sum(state.to_vec(), &mut self.sum_ids),
                    &mut self.sums,
                    |_| ControlFlow::Continue((0, None)),
                    |s, (i, moves)| {
                        while *i < s.len() {
                            if *i != 0 && s[*i - 1] == s[*i] {
                                *i += 1;
                                continue;
                            }
                            let iterator =
                                moves.get_or_insert_with(|| (self.moves)(&s[*i]).into_iter());
                            if let Some(next) = iterator.next() {
                                let mut to = s.clone();
                                to[*i] = next.to;
                                return Some(canonical_sum(to, &mut self.sum_ids));
                            }
                            *i += 1;
                            *moves = None;
                        }
                        None
                    },
                    true,
                )
            }
        } else {
            let (xor, odd) = self.reduce_sum(state);
            if xor == 0 && odd.len() == 1 {
                return self.outcome(odd.into_iter().next().unwrap(), false);
            }
            odd.into_iter().fold(xor, |value, s| value ^ self.grundy(s)) != 0
        };
        GameOutcome::from_win(win)
    }

    fn reduce_sum<'a>(&self, state: &'a [S]) -> (u64, HashSet<&'a S>) {
        let mut xor = 0;
        let mut odd = HashSet::new();
        for s in state {
            if let Some(&value) = self.grundy.get(s) {
                xor ^= value;
            } else if self.outcomes[0].get(s) != Some(&false) && !odd.insert(s) {
                odd.remove(s);
            }
        }
        (xor, odd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{Xorshift, testutil::sequence_cases};
    use std::cell::Cell;

    #[test]
    fn queries_and_caches() {
        let mut rng = Xorshift::default();
        let mut input_rng = Xorshift::default();
        for takes in sequence_cases(&mut input_rng, 1..=8, 0..=8) {
            let edges: Vec<Vec<_>> = (0usize..65)
                .map(|n| {
                    takes
                        .iter()
                        .filter_map(|&take| n.checked_sub(take).map(|to| Neighbor::new(to, take)))
                        .collect()
                })
                .collect();
            let calls = Cell::new(0);
            let mut game = GameAnalyzer::new().impartial_moves(|&n: &usize| {
                calls.set(calls.get() + 1);
                edges[n].iter().copied()
            });
            let mut grundy = [0u64; 65];
            let mut misere = [true; 65];
            let mut heights = [0usize; 65];
            for (n, moves) in edges.iter().enumerate() {
                grundy[n] = (0..)
                    .find(|g| moves.iter().all(|m| grundy[m.to] != *g))
                    .unwrap();
                misere[n] = moves.is_empty() || moves.iter().any(|m| !misere[m.to]);
                heights[n] = moves.iter().map(|m| heights[m.to] + 1).max().unwrap_or(0);
            }
            let states = [64, 0, 64, rng.random(0..65)];
            assert_eq!(game.grundy().table(states), states.map(|s| (s, grundy[s])));
            assert_eq!(game.grundy().verify(0..65, |&s| grundy[s]), None);
            assert_eq!(calls.get(), 65);
            let bad = rng.random(1..64);
            let later = rng.random(bad + 1..65);
            let mut checked = 0;
            let mismatch = game
                .grundy()
                .verify(0..65, |&s| {
                    checked += 1;
                    grundy[s] ^ u64::from(s == bad || s == later)
                })
                .unwrap();
            assert_eq!(
                (mismatch.state, mismatch.expected, mismatch.actual),
                (bad, grundy[bad] ^ 1, grundy[bad])
            );
            assert_eq!(calls.get(), 65);
            assert_eq!(checked, bad + 1);
            for (n, moves) in edges.iter().enumerate() {
                let [normal, misere_outcome] = [grundy[n] != 0, misere[n]].map(|win| {
                    if win {
                        GameOutcome::Win
                    } else {
                        GameOutcome::Lose
                    }
                });
                assert_eq!(game.outcome().eval(&n), normal);
                assert_eq!(game.misere().outcome().eval(&n), misere_outcome);
                assert_eq!(game.outcome().eval(&n), normal);
                assert_eq!(game.value().eval(&n).as_nimber(), Some(grundy[n]));
                let winning: Vec<_> = moves
                    .iter()
                    .filter(|m| grundy[m.to] == 0)
                    .copied()
                    .collect();
                assert_eq!(game.grundy().winning_moves(&n), winning);
                assert_eq!(game.value().winning_moves(&n), winning);
                assert_eq!(game.outcome().winning_moves(&n), winning);
                assert_eq!(
                    game.misere().outcome().winning_moves(&n),
                    moves
                        .iter()
                        .filter(|m| !misere[m.to])
                        .copied()
                        .collect::<Vec<_>>()
                );
            }
            let mut height = game.fold(|_: &usize, children: &[(usize, usize)]| {
                children.iter().map(|(_, h)| h + 1).max().unwrap_or(0)
            });
            assert_eq!(height.verify(0..65, |&s| heights[s]), None);
            for (n, moves) in edges.iter().enumerate() {
                let expected: Vec<_> = moves
                    .iter()
                    .map(|&next| EvaluatedMove {
                        next,
                        value: heights[next.to],
                    })
                    .collect();
                assert_eq!(height.moves(&n), expected);
            }
            drop(height);
            let mut order = game.fold(|_: &usize, children: &[(usize, Vec<usize>)]| {
                children
                    .iter()
                    .flat_map(|(label, path)| std::iter::once(*label).chain(path.iter().copied()))
                    .collect()
            });
            // A preorder traversal independently checks the noncommutative fold.
            for n in 0usize..=4 {
                let mut expected = Vec::new();
                let mut stack = vec![(n, 0)];
                while let Some((s, label)) = stack.pop() {
                    if label != 0 {
                        expected.push(label);
                    }
                    stack.extend(edges[s].iter().rev().map(|m| (m.to, m.label)));
                }
                assert_eq!(order.eval(&n), expected);
            }
            drop(order);
            game.clear_cache();
            let before = calls.get();
            assert_eq!(game.grundy().verify(0..65, |&s| grundy[s]), None);
            assert_eq!(calls.get() - before, 65);
            game.clear_cache();
            // Outcome-first queries must agree before Grundy has filled the cache.
            for n in (0..65).rev() {
                assert_eq!(game.outcome().eval(&n) == GameOutcome::Win, grundy[n] != 0);
                assert_eq!(
                    game.misere().outcome().eval(&n) == GameOutcome::Win,
                    misere[n]
                );
                assert_eq!(game.grundy().eval(&n), grundy[n]);
            }
        }
    }

    #[test]
    fn cached_children() {
        for n in [0usize, 1, 32, 33] {
            let calls = Cell::new(0);
            let mut game = GameAnalyzer::new().impartial(|&s: &usize| {
                calls.set(calls.get() + 1);
                s.checked_sub(1)
            });
            game.grundy().eval(&n);
            calls.set(0);
            let normal = if n % 2 == 0 {
                GameOutcome::Win
            } else {
                GameOutcome::Lose
            };
            assert_eq!(game.outcome().eval(&(n + 1)), normal);
            assert_eq!(calls.get(), 1);
            calls.set(0);
            let misere = if n % 2 == 0 {
                GameOutcome::Lose
            } else {
                GameOutcome::Win
            };
            assert_eq!(game.misere().outcome().eval(&(n + 1)), misere);
            assert_eq!(calls.get(), n + 2);
            game.clear_cache();
            game.outcome().eval(&n);
            calls.set(0);
            assert_eq!(game.grundy().eval(&(n + 1)), (n as u64 + 1) % 2);
            assert_eq!(calls.get(), 1 + n % 2);
        }
    }

    #[test]
    fn shortcuts() {
        for n in [1usize, 2, 31, 32] {
            let transitions = Cell::new(0);
            let mut game = GameAnalyzer::new().impartial_moves(|&s: &usize| {
                let transitions = &transitions;
                (0..s).map(move |to| {
                    transitions.set(transitions.get() + 1);
                    Neighbor::new(to, s - to)
                })
            });
            assert_eq!(game.sum().grundy().eval(&[n, n]), 0);
            assert_eq!(game.sum().outcome().eval(&[n, n]), GameOutcome::Lose);
            assert_eq!(transitions.get(), 0);
            for query in 0..3 {
                let winning = match query {
                    0 => game.grundy().winning_moves(&n),
                    1 => game.value().winning_moves(&n),
                    _ => game.outcome().winning_moves(&n),
                };
                assert_eq!(winning, vec![Neighbor::new(0, n)]);
                assert_eq!(transitions.get(), 2 * n - 1);
                game.clear_cache();
                transitions.set(0);
            }
            assert_eq!(
                game.sum().outcome().winning_moves(&[n]),
                vec![Neighbor::new(vec![0], (0, n))]
            );
            assert_eq!(transitions.get(), 2 * n - 1);
            game.clear_cache();
            transitions.set(0);
            assert_eq!(game.sum().outcome().eval(&[n, 1, n]), GameOutcome::Win);
            assert_eq!(transitions.get(), 1);
            assert_eq!(game.sum().grundy().eval(&[n, 1, n]), 1);
        }
    }
}
