use super::{
    ControlFlow, EvaluatedMove, GameAnalysis, GameAnalyzer, GameSum, GameValue, Hash, HashMap,
    HashSet, Misere, Neighbor, PartisanOutcome, Player, canonical_sum, memoized_game,
    memoized_wins,
};
use std::cmp::Ordering;

pub struct Partisan<S, FL, FR> {
    left: FL,
    right: FR,
    values: HashMap<S, GameValue>,
    outcomes: [HashMap<(S, Player), bool>; 2],
    sums: [HashMap<(Vec<S>, Player), bool>; 2],
    sum_ids: HashMap<S, usize>,
}

impl GameAnalyzer {
    #[allow(clippy::type_complexity)] // Each adapter preserves the caller's iterator.
    pub fn partisan<S, FL, FR, IL, IR>(
        self,
        mut left: FL,
        mut right: FR,
    ) -> GameAnalyzer<
        Partisan<
            S,
            impl FnMut(&S) -> std::iter::Map<IL::IntoIter, fn(S) -> Neighbor<S, ()>>,
            impl FnMut(&S) -> std::iter::Map<IR::IntoIter, fn(S) -> Neighbor<S, ()>>,
        >,
    >
    where
        FL: FnMut(&S) -> IL,
        FR: FnMut(&S) -> IR,
        IL: IntoIterator<Item = S>,
        IR: IntoIterator<Item = S>,
    {
        let label: fn(S) -> Neighbor<S, ()> = |to| Neighbor::new(to, ());
        self.partisan_moves(
            move |s: &S| left(s).into_iter().map(label),
            move |s: &S| right(s).into_iter().map(label),
        )
    }

    pub fn partisan_moves<S, L, FL, FR, IL, IR>(
        self,
        left: FL,
        right: FR,
    ) -> GameAnalyzer<Partisan<S, FL, FR>>
    where
        FL: FnMut(&S) -> IL,
        FR: FnMut(&S) -> IR,
        IL: IntoIterator<Item = Neighbor<S, L>>,
        IR: IntoIterator<Item = Neighbor<S, L>>,
    {
        GameAnalyzer {
            game: Partisan {
                left,
                right,
                values: HashMap::new(),
                outcomes: std::array::from_fn(|_| HashMap::new()),
                sums: std::array::from_fn(|_| HashMap::new()),
                sum_ids: HashMap::new(),
            },
        }
    }
}

impl<S, FL, FR> GameAnalyzer<Partisan<S, FL, FR>> {
    pub fn misere(&mut self) -> Misere<'_, Partisan<S, FL, FR>> {
        Misere {
            game: &mut self.game,
        }
    }

    pub fn sum(&mut self) -> GameSum<'_, Partisan<S, FL, FR>> {
        GameSum {
            game: &mut self.game,
        }
    }

    pub fn clear_cache(&mut self) {
        self.game.values.clear();
        self.game.outcomes.iter_mut().for_each(HashMap::clear);
        self.game.sums.iter_mut().for_each(HashMap::clear);
        self.game.sum_ids.clear();
    }
}

impl<S, L, FL, FR, IL, IR> GameAnalyzer<Partisan<S, FL, FR>>
where
    S: Clone + Eq + Hash,
    FL: FnMut(&S) -> IL,
    FR: FnMut(&S) -> IR,
    IL: IntoIterator<Item = Neighbor<S, L>>,
    IR: IntoIterator<Item = Neighbor<S, L>>,
{
    pub fn value(&mut self) -> GameAnalysis<'_, Partisan<S, FL, FR>, S, GameValue> {
        GameAnalysis::new(&mut self.game, Partisan::value)
    }

    pub fn outcome(&mut self) -> GameAnalysis<'_, Partisan<S, FL, FR>, S, PartisanOutcome> {
        GameAnalysis::new(&mut self.game, |g, s| g.outcome(s, false))
    }

    /// `combine` receives all Left options followed by all Right options.
    #[allow(clippy::type_complexity)] // The query owns the fold and its cache.
    pub fn fold<V, C>(
        &mut self,
        mut combine: C,
    ) -> GameAnalysis<
        '_,
        Partisan<S, FL, FR>,
        S,
        V,
        false,
        impl FnMut(&mut Partisan<S, FL, FR>, &S) -> V,
    >
    where
        V: Clone,
        C: FnMut(&S, &[(Player, L, V)]) -> V,
    {
        let mut cache = HashMap::new();
        GameAnalysis::new(
            &mut self.game,
            move |game: &mut Partisan<S, FL, FR>, state: &S| {
                memoized_game(
                    state,
                    &mut cache,
                    &mut HashSet::new(),
                    &mut |s| {
                        let moves = (game.left)(s)
                            .into_iter()
                            .map(|n| (n.to, (Player::Left, n.label)))
                            .chain(
                                (game.right)(s)
                                    .into_iter()
                                    .map(|n| (n.to, (Player::Right, n.label))),
                            );
                        ControlFlow::Continue((moves, Vec::new()))
                    },
                    &mut |_, moves| moves.next(),
                    &mut |values, (player, label), value| {
                        values.push((player, label, value));
                        ControlFlow::Continue(())
                    },
                    &mut |s, values| combine(s, &values),
                )
            },
        )
    }
}
impl<'a, S, L, FL, FR, IL, IR> Misere<'a, Partisan<S, FL, FR>>
where
    S: Clone + Eq + Hash,
    FL: FnMut(&S) -> IL,
    FR: FnMut(&S) -> IR,
    IL: IntoIterator<Item = Neighbor<S, L>>,
    IR: IntoIterator<Item = Neighbor<S, L>>,
{
    pub fn outcome(self) -> GameAnalysis<'a, Partisan<S, FL, FR>, S, PartisanOutcome, true> {
        GameAnalysis::new(self.game, |g, s| g.outcome(s, true))
    }
}
impl<'a, S, L, FL, FR, IL, IR> GameSum<'a, Partisan<S, FL, FR>>
where
    S: Clone + Eq + Hash,
    FL: FnMut(&S) -> IL,
    FR: FnMut(&S) -> IR,
    IL: IntoIterator<Item = Neighbor<S, L>>,
    IR: IntoIterator<Item = Neighbor<S, L>>,
{
    pub fn value(self) -> GameAnalysis<'a, Partisan<S, FL, FR>, [S], GameValue> {
        GameAnalysis::new(self.game, |g, state: &[S]| {
            state
                .iter()
                .fold(GameValue::from(0), |value, s| value + g.value(s))
        })
    }
}
impl<'a, S, L, FL, FR, IL, IR, const MISERE: bool> GameSum<'a, Partisan<S, FL, FR>, MISERE>
where
    S: Clone + Eq + Hash,
    FL: FnMut(&S) -> IL,
    FR: FnMut(&S) -> IR,
    IL: IntoIterator<Item = Neighbor<S, L>>,
    IR: IntoIterator<Item = Neighbor<S, L>>,
{
    /// Explores reachable sum positions, including the player to move.
    /// Reuses cached number-plus-nimber values in normal play.
    pub fn outcome(self) -> GameAnalysis<'a, Partisan<S, FL, FR>, [S], PartisanOutcome, MISERE> {
        GameAnalysis::new(self.game, |g, s| {
            match (
                g.sum_wins(s, Player::Left, MISERE),
                g.sum_wins(s, Player::Right, MISERE),
            ) {
                (true, true) => PartisanOutcome::Next,
                (false, false) => PartisanOutcome::Previous,
                (true, false) => PartisanOutcome::Left,
                (false, true) => PartisanOutcome::Right,
            }
        })
    }
}

impl<S, L, FL, FR, IL, IR, V, const MISERE: bool, E>
    GameAnalysis<'_, Partisan<S, FL, FR>, S, V, MISERE, E>
where
    FL: FnMut(&S) -> IL,
    FR: FnMut(&S) -> IR,
    IL: IntoIterator<Item = Neighbor<S, L>>,
    IR: IntoIterator<Item = Neighbor<S, L>>,
    E: FnMut(&mut Partisan<S, FL, FR>, &S) -> V,
{
    /// Values describe the destination game for both choices of starting player.
    pub fn moves(&mut self, state: &S, player: Player) -> Vec<EvaluatedMove<S, L, V>> {
        partisan_moves(&mut self.game.left, &mut self.game.right, state, player)
            .map(|next| {
                let value = self.eval(&next.to);
                EvaluatedMove { next, value }
            })
            .collect()
    }
}

impl<S, L, FL, FR, IL, IR> GameAnalysis<'_, Partisan<S, FL, FR>, S, GameValue>
where
    FL: FnMut(&S) -> IL,
    FR: FnMut(&S) -> IR,
    IL: IntoIterator<Item = Neighbor<S, L>>,
    IR: IntoIterator<Item = Neighbor<S, L>>,
{
    pub fn compare(&mut self, left: &S, right: &S) -> Option<Ordering> {
        self.eval(left).partial_cmp(&self.eval(right))
    }

    /// Undominated moves for the selected player; incomparable alternatives remain.
    pub fn undominated_moves(&mut self, state: &S, player: Player) -> Vec<Neighbor<S, L>> {
        let moves = self.moves(state, player);
        let preferred = if player == Player::Left {
            Ordering::Greater
        } else {
            Ordering::Less
        };
        let number = moves.iter().try_fold(None, |best, m| {
            let (q, _) = m.value.as_number_nimber()?;
            Some(Some(
                best.map_or(q, |b| if q.cmp(&b) == preferred { q } else { b }),
            ))
        });
        if let Some(best) = number {
            return moves
                .into_iter()
                .filter(|m| m.value.as_number_nimber().map(|(q, _)| q) == best)
                .map(|m| m.next)
                .collect();
        }
        let mut dominated = vec![false; moves.len()];
        for i in 0..moves.len() {
            if dominated[i] {
                continue;
            }
            for j in i + 1..moves.len() {
                if dominated[j] {
                    continue;
                }
                let order = moves[i].value.partial_cmp(&moves[j].value);
                if order == Some(preferred) {
                    dominated[j] = true;
                } else if order == Some(preferred.reverse()) {
                    dominated[i] = true;
                    break;
                }
            }
        }
        moves
            .into_iter()
            .zip(dominated)
            .filter(|(_, d)| !d)
            .map(|(m, _)| m.next)
            .collect()
    }
}
impl<S, L, FL, FR, IL, IR, const MISERE: bool>
    GameAnalysis<'_, Partisan<S, FL, FR>, S, PartisanOutcome, MISERE>
where
    S: Clone + Eq + Hash,
    FL: FnMut(&S) -> IL,
    FR: FnMut(&S) -> IR,
    IL: IntoIterator<Item = Neighbor<S, L>>,
    IR: IntoIterator<Item = Neighbor<S, L>>,
{
    /// Whether the selected starting player can force a win.
    pub fn wins(&mut self, state: &S, player: Player) -> bool {
        self.game.wins(state, player, MISERE)
    }

    pub fn winning_moves(&mut self, state: &S, player: Player) -> Vec<Neighbor<S, L>> {
        partisan_moves(&mut self.game.left, &mut self.game.right, state, player)
            .filter(|next| !self.game.wins(&next.to, player.opponent(), MISERE))
            .collect()
    }
}

#[allow(clippy::type_complexity)] // Sum states and labels both carry component information.
impl<S, L, FL, FR, IL, IR, V, const MISERE: bool, E>
    GameAnalysis<'_, Partisan<S, FL, FR>, [S], V, MISERE, E>
where
    S: Clone,
    FL: FnMut(&S) -> IL,
    FR: FnMut(&S) -> IR,
    IL: IntoIterator<Item = Neighbor<S, L>>,
    IR: IntoIterator<Item = Neighbor<S, L>>,
    E: FnMut(&mut Partisan<S, FL, FR>, &[S]) -> V,
{
    pub fn moves(
        &mut self,
        state: &[S],
        player: Player,
    ) -> Vec<EvaluatedMove<Vec<S>, (usize, L), V>> {
        let mut result = Vec::new();
        for (i, component) in state.iter().enumerate() {
            for next in partisan_moves(&mut self.game.left, &mut self.game.right, component, player)
            {
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
impl<S, L, FL, FR, IL, IR, const MISERE: bool>
    GameAnalysis<'_, Partisan<S, FL, FR>, [S], PartisanOutcome, MISERE>
where
    S: Clone + Eq + Hash,
    FL: FnMut(&S) -> IL,
    FR: FnMut(&S) -> IR,
    IL: IntoIterator<Item = Neighbor<S, L>>,
    IR: IntoIterator<Item = Neighbor<S, L>>,
{
    /// Whether the selected starting player can force a win.
    pub fn wins(&mut self, state: &[S], player: Player) -> bool {
        self.game.sum_wins(state, player, MISERE)
    }

    pub fn winning_moves(
        &mut self,
        state: &[S],
        player: Player,
    ) -> Vec<Neighbor<Vec<S>, (usize, L)>> {
        let mut result = Vec::new();
        for (i, component) in state.iter().enumerate() {
            for next in partisan_moves(&mut self.game.left, &mut self.game.right, component, player)
            {
                let mut to = state.to_vec();
                to[i] = next.to;
                if !self.game.sum_wins(&to, player.opponent(), MISERE) {
                    result.push(Neighbor::new(to, (i, next.label)));
                }
            }
        }
        result
    }
}

impl<S, L, FL, FR, IL, IR> Partisan<S, FL, FR>
where
    S: Clone + Eq + Hash,
    FL: FnMut(&S) -> IL,
    FR: FnMut(&S) -> IR,
    IL: IntoIterator<Item = Neighbor<S, L>>,
    IR: IntoIterator<Item = Neighbor<S, L>>,
{
    fn value(&mut self, state: &S) -> GameValue {
        memoized_game(
            state,
            &mut self.values,
            &mut HashSet::new(),
            &mut |s| {
                let moves = (self.left)(s)
                    .into_iter()
                    .map(|n| (n.to, Player::Left))
                    .chain((self.right)(s).into_iter().map(|n| (n.to, Player::Right)));
                ControlFlow::Continue((moves, (Vec::new(), Vec::new())))
            },
            &mut |_, moves| moves.next(),
            &mut |(left, right), player, value| {
                if player == Player::Left {
                    left.push(value)
                } else {
                    right.push(value)
                }
                ControlFlow::Continue(())
            },
            &mut |_, (left, right)| GameValue::from_options(left, right),
        )
    }

    fn outcome(&mut self, state: &S, misere: bool) -> PartisanOutcome {
        match (
            self.wins(state, Player::Left, misere),
            self.wins(state, Player::Right, misere),
        ) {
            (true, true) => PartisanOutcome::Next,
            (false, false) => PartisanOutcome::Previous,
            (true, false) => PartisanOutcome::Left,
            (false, true) => PartisanOutcome::Right,
        }
    }

    fn wins(&mut self, state: &S, player: Player, misere: bool) -> bool {
        let key = (state.clone(), player);
        memoized_wins(
            &key,
            &mut self.outcomes[usize::from(misere)],
            |(s, p)| {
                if !misere && let Some(value) = self.values.get(s) {
                    return ControlFlow::Break(value.wins(*p));
                }
                let next_player = p.opponent();
                let moves = partisan_moves(&mut self.left, &mut self.right, s, *p)
                    .map(move |n| (n.to, next_player));
                ControlFlow::Continue(moves)
            },
            |_, moves| moves.next(),
            misere,
        )
    }

    fn sum_wins(&mut self, state: &[S], player: Player, misere: bool) -> bool {
        match state {
            [] => return misere,
            [single] => return self.wins(single, player, misere),
            _ => {}
        }
        let mut normalize = |mut state: Vec<S>| {
            if !misere {
                state.retain(|s| {
                    if let Some(value) = self.values.get(s) {
                        return value.as_nimber() != Some(0);
                    }
                    let mut key = (s.clone(), Player::Left);
                    if self.outcomes[0].get(&key) != Some(&false) {
                        return true;
                    }
                    key.1 = Player::Right;
                    self.outcomes[0].get(&key) != Some(&false)
                });
            }
            canonical_sum(state, &mut self.sum_ids)
        };
        let state = normalize(state.to_vec());
        match state.as_slice() {
            [] => return misere,
            [single] => return self.wins(single, player, misere),
            _ => {}
        }
        let key = (state, player);
        memoized_wins(
            &key,
            &mut self.sums[usize::from(misere)],
            |(s, p)| {
                if !misere
                    && let Some(value) = s.iter().try_fold(GameValue::from(0), |sum, s| {
                        self.values
                            .get(s)
                            .filter(|g| g.as_number_nimber().is_some())
                            .map(|g| sum + g.clone())
                    })
                {
                    return ControlFlow::Break(value.wins(*p));
                }
                // Losing as starter in every component implies losing in the sum.
                // One known win is enough if the opponent loses in every other component.
                if !misere && (!self.values.is_empty() || !self.outcomes[0].is_empty()) {
                    let mut all_lose = true;
                    let mut all_opponent_lose = true;
                    let mut can_win = false;
                    for s in s {
                        let (mut win, mut opponent_win) = (None, None);
                        if !self.outcomes[0].is_empty() {
                            let mut key = (s.clone(), *p);
                            win = self.outcomes[0].get(&key).copied();
                            key.1 = p.opponent();
                            opponent_win = self.outcomes[0].get(&key).copied();
                        }
                        if let Some(value) = self.values.get(s) {
                            win.get_or_insert_with(|| value.wins(*p));
                            opponent_win.get_or_insert_with(|| value.wins(p.opponent()));
                        }
                        can_win = can_win && opponent_win == Some(false)
                            || all_opponent_lose && win == Some(true);
                        all_lose &= win == Some(false);
                        all_opponent_lose &= opponent_win == Some(false);
                        if !all_lose && !all_opponent_lose && !can_win {
                            break;
                        }
                    }
                    if all_lose || can_win {
                        return ControlFlow::Break(can_win);
                    }
                }
                ControlFlow::Continue((0, None))
            },
            |(s, p), (i, moves)| {
                while *i < s.len() {
                    if *i != 0 && s[*i - 1] == s[*i] {
                        *i += 1;
                        continue;
                    }
                    let iterator = moves.get_or_insert_with(|| {
                        partisan_moves(&mut self.left, &mut self.right, &s[*i], *p)
                    });
                    if let Some(next) = iterator.next() {
                        let mut to = s.clone();
                        to[*i] = next.to;
                        return Some((normalize(to), p.opponent()));
                    }
                    *i += 1;
                    *moves = None;
                }
                None
            },
            misere,
        )
    }
}

fn partisan_moves<S, L, FL, FR, IL, IR>(
    left: &mut FL,
    right: &mut FR,
    state: &S,
    player: Player,
) -> impl Iterator<Item = Neighbor<S, L>> + use<S, L, FL, FR, IL, IR>
where
    FL: FnMut(&S) -> IL,
    FR: FnMut(&S) -> IR,
    IL: IntoIterator<Item = Neighbor<S, L>>,
    IR: IntoIterator<Item = Neighbor<S, L>>,
{
    let (left, right) = match player {
        Player::Left => (Some(left(state).into_iter()), None),
        Player::Right => (None, Some(right(state).into_iter())),
    };
    left.into_iter()
        .flatten()
        .chain(right.into_iter().flatten())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::testutil::exhaustive_sequences;
    use std::cell::Cell;

    #[test]
    fn cached_children() {
        for n in [0usize, 1, 32, 33] {
            let calls = Cell::new(0);
            let mut game = GameAnalyzer::new().partisan(
                |&s: &usize| {
                    calls.set(calls.get() + 1);
                    s.checked_sub(1)
                },
                |&s: &usize| {
                    calls.set(calls.get() + 1);
                    s.checked_sub(1)
                },
            );
            game.value().eval(&n);
            for player in [Player::Left, Player::Right] {
                calls.set(0);
                assert_eq!(game.outcome().wins(&(n + 1), player), n % 2 == 0);
                assert_eq!(calls.get(), 1);
                calls.set(0);
                assert_eq!(game.sum().outcome().wins(&[n + 1, 0], player), n % 2 == 0);
                // A losing root may also enumerate the terminal component.
                assert!(calls.get() <= 2);
            }
            calls.set(0);
            assert_eq!(
                game.misere().outcome().wins(&(n + 1), Player::Left),
                n % 2 != 0
            );
            assert_eq!(calls.get(), n + 2);
        }
    }

    #[test]
    fn values_and_fold() {
        type Trace = (usize, Vec<(Player, usize)>);

        let basic = basic_games();
        let mut rng = crate::tools::Xorshift::default();
        for moves in crate::tools::testutil::structured_sequences(&mut rng, 0..basic.len(), 0..=12)
        {
            let mut options = basic.clone();
            options.push([moves.clone(), moves]);
            let n = options.len();
            // Direct play on G-H establishes order independently of GameValue.
            let mut differences = vec![vec![[false; 2]; n]; n];
            for a in 0..n {
                for b in 0..n {
                    for p in 0..2 {
                        differences[a][b][p] =
                            options[a][p].iter().any(|&to| !differences[to][b][p ^ 1])
                                || options[b][p ^ 1]
                                    .iter()
                                    .any(|&to| !differences[a][to][p ^ 1]);
                    }
                }
            }
            let labeled_moves = |s: usize, p: usize| {
                options[s][p]
                    .iter()
                    .enumerate()
                    .map(|(label, &to)| Neighbor::new(to, label))
            };
            let mut game = GameAnalyzer::new()
                .partisan_moves(|&s| labeled_moves(s, 0), |&s| labeled_moves(s, 1));
            let values: Vec<_> = game
                .value()
                .table(0..n)
                .into_iter()
                .map(|(_, v)| v)
                .collect();
            let notation = ["0", "1", "-1", "*", "*2", "↑", "↓", "{1|-1}", "1/2", "0"];
            assert_eq!(
                values[..basic.len()]
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
                notation
            );
            game.clear_cache();
            assert_eq!(game.value().verify(0..n, |&s| values[s].clone()), None);
            for (a, row) in differences.iter().enumerate() {
                for (b, &wins) in row.iter().enumerate() {
                    let expected = match wins {
                        [true, true] => None,
                        [false, false] => Some(Ordering::Equal),
                        [true, false] => Some(Ordering::Greater),
                        [false, true] => Some(Ordering::Less),
                    };
                    assert_eq!(game.value().compare(&a, &b), expected);
                }
                for (p, player) in [Player::Left, Player::Right].into_iter().enumerate() {
                    let expected: Vec<_> = labeled_moves(a, p)
                        .map(|next| EvaluatedMove {
                            value: values[next.to].clone(),
                            next,
                        })
                        .collect();
                    assert_eq!(game.value().moves(&a, player), expected);
                    let undominated: Vec<_> = expected
                        .into_iter()
                        .filter(|m| {
                            !options[a][p].iter().any(|&b| {
                                differences[b][m.next.to][p] && !differences[b][m.next.to][p ^ 1]
                            })
                        })
                        .map(|m| m.next)
                        .collect();
                    assert_eq!(game.value().undominated_moves(&a, player), undominated);
                }
            }
            assert_eq!(game.sum().value().eval(&[5, 6]), GameValue::from(0));
            assert_eq!(
                game.sum().value().moves(&[1, 2], Player::Left),
                vec![EvaluatedMove {
                    next: Neighbor::new(vec![0, 2], (0, 0)),
                    value: GameValue::from(-1),
                }]
            );
            let mut fold = game.fold(|_: &usize, children: &[(Player, usize, Trace)]| {
                let height = children
                    .iter()
                    .map(|(_, _, (h, _))| h + 1)
                    .max()
                    .unwrap_or(0);
                let order: Vec<_> = children
                    .iter()
                    .flat_map(|(player, label, (_, path))| {
                        std::iter::once((*player, *label)).chain(path.iter().copied())
                    })
                    .collect();
                (height, order)
            });
            // Direct traversal checks height, side order and move labels together.
            let expected: Vec<Trace> = (0..n)
                .map(|s| {
                    let mut height = 0;
                    let mut order = Vec::new();
                    let mut stack = vec![(s, 0, None)];
                    while let Some((s, depth, edge)) = stack.pop() {
                        height = height.max(depth);
                        order.extend(edge);
                        for (p, player) in
                            [Player::Left, Player::Right].into_iter().enumerate().rev()
                        {
                            for next in labeled_moves(s, p).rev() {
                                stack.push((next.to, depth + 1, Some((player, next.label))));
                            }
                        }
                    }
                    (height, order)
                })
                .collect();
            assert_eq!(fold.verify(0..n, |&s| expected[s].clone()), None);
            for s in 0..n {
                for (p, player) in [Player::Left, Player::Right].into_iter().enumerate() {
                    let moves: Vec<_> = labeled_moves(s, p)
                        .map(|next| EvaluatedMove {
                            value: expected[next.to].clone(),
                            next,
                        })
                        .collect();
                    assert_eq!(fold.moves(&s, player), moves);
                }
            }
        }
    }

    #[test]
    fn sums_against_play() {
        // Basic values, every subtraction limit through three, and DAGs of each density.
        let mut games = vec![(basic_games(), 3)];
        for left in 0..=3usize {
            for right in 0..=3usize {
                games.push((
                    (0..5usize)
                        .map(|s| {
                            [
                                (s.saturating_sub(left)..s).collect(),
                                (s.saturating_sub(right)..s).collect(),
                            ]
                        })
                        .collect(),
                    4,
                ));
            }
        }
        let mut rng = crate::tools::Xorshift::default();
        for n in 0..=8 {
            for degree in 0..=n {
                let options = (0..n)
                    .map(|s| {
                        std::array::from_fn(|_| {
                            (0..if s == 0 { 0 } else { degree })
                                .map(|_| rng.rand(s as u64) as usize)
                                .collect()
                        })
                    })
                    .collect();
                games.push((options, 3));
            }
        }
        let classify = |wins: [bool; 2]| match wins {
            [true, true] => PartisanOutcome::Next,
            [false, false] => PartisanOutcome::Previous,
            [true, false] => PartisanOutcome::Left,
            [false, true] => PartisanOutcome::Right,
        };
        for (options, max_len) in games {
            let n = options.len();
            let mut local = vec![[false; 2]; n];
            for s in 0..n {
                for p in 0..2 {
                    local[s][p] = options[s][p].iter().any(|&to| !local[to][p ^ 1]);
                }
            }
            for len in 0..=max_len {
                let states: Vec<_> = exhaustive_sequences(0..n, len..=len).collect();
                let tables = [false, true].map(|misere| {
                    // Ordered product DP without value arithmetic, sorting or cache shortcuts.
                    let mut wins = vec![[misere; 2]; states.len()];
                    for (id, state) in states.iter().enumerate() {
                        for p in 0..2 {
                            let mut children = Vec::new();
                            let mut place = 1;
                            for &s in state {
                                children
                                    .extend(options[s][p].iter().map(|to| id - (s - to) * place));
                                place *= n;
                            }
                            if !children.is_empty() {
                                wins[id][p] = children.iter().any(|&to| !wins[to][p ^ 1]);
                            }
                        }
                    }
                    wins
                });
                let variants = (0..6)
                    .map(|cache| (cache, true))
                    .chain([(0, false), (2, false)]);
                for (cache, tagged) in variants {
                    let calls = Cell::new(0);
                    let mut game = GameAnalyzer::new().partisan_moves(
                        |&(i, s): &(usize, usize)| {
                            calls.set(calls.get() + 1);
                            options[s][0]
                                .iter()
                                .enumerate()
                                .map(move |(label, &to)| Neighbor::new((i, to), label))
                        },
                        |&(i, s): &(usize, usize)| {
                            calls.set(calls.get() + 1);
                            options[s][1]
                                .iter()
                                .enumerate()
                                .map(move |(label, &to)| Neighbor::new((i, to), label))
                        },
                    );
                    for i in 0..len {
                        for s in 0..n {
                            match cache {
                                1 => {
                                    game.value().eval(&(i, s));
                                }
                                2 => {
                                    game.outcome().eval(&(i, s));
                                }
                                3 => {
                                    game.outcome().wins(&(i, s), Player::Left);
                                }
                                4 => {
                                    game.outcome().wins(&(i, s), Player::Right);
                                    if i == 0 {
                                        game.outcome().wins(&(i, s), Player::Left);
                                    }
                                }
                                5 if s % 2 == 0 => {
                                    game.outcome().wins(&(i, s), Player::Left);
                                }
                                _ => {}
                            }
                        }
                    }
                    for (misere, wins) in [false, true].into_iter().zip(&tables) {
                        for state in states.iter().rev() {
                            let mut components: Vec<_> = state
                                .iter()
                                .copied()
                                .enumerate()
                                .map(|(i, s)| (if tagged { i } else { 0 }, s))
                                .collect();
                            for reversed in [false, true] {
                                if reversed {
                                    components.reverse();
                                }
                                let id = components.iter().rev().fold(0, |id, &(_, s)| id * n + s);
                                for (p, player) in
                                    [Player::Left, Player::Right].into_iter().enumerate()
                                {
                                    calls.set(0);
                                    let actual = if misere {
                                        game.sum().misere().outcome().wins(&components, player)
                                    } else {
                                        game.sum().outcome().wins(&components, player)
                                    };
                                    assert_eq!(
                                        actual, wins[id][p],
                                        "{options:?}, {state:?}, {player:?}, {misere}"
                                    );
                                    let all_lose = state.iter().all(|&s| !local[s][p]);
                                    let all_opponent_lose = state.iter().all(|&s| !local[s][p ^ 1]);
                                    let known = match cache {
                                        1 | 2 => {
                                            all_lose
                                                || state.iter().enumerate().any(|(i, &s)| {
                                                    local[s][p]
                                                        && state.iter().enumerate().all(
                                                            |(j, &t)| i == j || !local[t][p ^ 1],
                                                        )
                                                })
                                                || state
                                                    .iter()
                                                    .filter(|&&s| local[s] != [false; 2])
                                                    .count()
                                                    <= 1
                                        }
                                        3 => p == 0 && all_lose,
                                        4 => {
                                            p == 1 && all_lose
                                                || p == 0
                                                    && all_opponent_lose
                                                    && state.first().is_some_and(|&s| local[s][0])
                                        }
                                        _ => false,
                                    };
                                    if !misere && known {
                                        assert_eq!(
                                            calls.get(),
                                            0,
                                            "cache={cache}, {state:?}, {player:?}"
                                        );
                                    }
                                    let mut expected = Vec::new();
                                    let mut place = 1;
                                    for (i, &(_, s)) in components.iter().enumerate() {
                                        for (label, &to) in options[s][p].iter().enumerate() {
                                            let mut next = components.clone();
                                            next[i].1 = to;
                                            expected.push((
                                                Neighbor::new(next, (i, label)),
                                                wins[id - (s - to) * place],
                                            ));
                                        }
                                        place *= n;
                                    }
                                    let moves = if misere {
                                        game.sum()
                                            .misere()
                                            .outcome()
                                            .winning_moves(&components, player)
                                    } else {
                                        game.sum().outcome().winning_moves(&components, player)
                                    };
                                    let winning: Vec<_> = expected
                                        .iter()
                                        .filter(|(_, win)| !win[p ^ 1])
                                        .map(|(next, _)| next.clone())
                                        .collect();
                                    assert_eq!(moves, winning);
                                    // Full component caches keep these extra queries out of partial-cache checks.
                                    if cache == 2 {
                                        let moves = if misere {
                                            game.sum().misere().outcome().moves(&components, player)
                                        } else {
                                            game.sum().outcome().moves(&components, player)
                                        };
                                        let evaluated: Vec<_> = expected
                                            .iter()
                                            .map(|(next, win)| EvaluatedMove {
                                                next: next.clone(),
                                                value: classify(*win),
                                            })
                                            .collect();
                                        assert_eq!(moves, evaluated);
                                    }
                                    if let [single] = components.as_slice() {
                                        let (win, moves) = if misere {
                                            let mut analysis = game.misere().outcome();
                                            (
                                                analysis.wins(single, player),
                                                analysis.winning_moves(single, player),
                                            )
                                        } else {
                                            let mut analysis = game.outcome();
                                            (
                                                analysis.wins(single, player),
                                                analysis.winning_moves(single, player),
                                            )
                                        };
                                        assert_eq!(win, wins[id][p]);
                                        let expected: Vec<_> = winning
                                            .into_iter()
                                            .map(|next| Neighbor::new(next.to[0], next.label.1))
                                            .collect();
                                        assert_eq!(moves, expected);
                                    }
                                }
                                let expected = classify(wins[id]);
                                let actual = if misere {
                                    game.misere().sum().outcome().eval(&components)
                                } else {
                                    game.sum().outcome().eval(&components)
                                };
                                assert_eq!(actual, expected);
                                if let [single] = components.as_slice() {
                                    let actual = if misere {
                                        game.misere().outcome().eval(single)
                                    } else {
                                        game.outcome().eval(single)
                                    };
                                    assert_eq!(actual, expected);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn integers() {
        let mut integers = GameAnalyzer::new().partisan(
            |&n: &i32| if n > 0 { Some(n - 1) } else { None },
            |&n: &i32| if n < 0 { Some(n + 1) } else { None },
        );
        for n in (0..=8192).flat_map(|n| [n, -n]) {
            assert_eq!(
                integers.value().eval(&n).as_number(),
                Some(i64::from(n).into())
            );
        }
    }

    fn basic_games() -> Vec<[Vec<usize>; 2]> {
        // Numbers, nimbers, infinitesimals, a switch and a nonterminal zero.
        vec![
            [vec![], vec![]],
            [vec![0], vec![]],
            [vec![], vec![0]],
            [vec![0], vec![0]],
            [vec![0, 3], vec![0, 3]],
            [vec![0], vec![3]],
            [vec![3], vec![0]],
            [vec![1], vec![2]],
            [vec![0], vec![1]],
            [vec![3], vec![3]],
        ]
    }
}
