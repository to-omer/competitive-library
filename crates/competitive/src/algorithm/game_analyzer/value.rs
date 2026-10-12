use super::{Dyadic, PartisanOutcome, Player, game_mex};
use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt,
    ops::{Add, Neg, Sub},
    rc::Rc,
};

/// A canonical finite normal-play game. Numbers use `Dyadic<i64>`.
/// `PartialOrd` is the game order: `None` means fuzzy, not a numeric approximation.
///
/// ```
/// use competitive::algorithm::GameValue;
/// let zero = GameValue::from(0);
/// let star = GameValue::nimber(1);
/// let up = GameValue::from_options([zero.clone()], [star.clone()]);
/// assert!(up > zero);
/// assert_eq!(star.partial_cmp(&zero), None);
/// assert_eq!(star.clone() + star, zero);
/// ```
#[derive(Clone)]
pub struct GameValue(GameValueNode);

#[derive(Clone)]
enum GameValueNode {
    NumberNimber(Dyadic<i64>, u64),
    Options(Rc<(Vec<GameValue>, Vec<GameValue>)>),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
enum GameValueKey {
    NumberNimber(Dyadic<i64>, u64),
    Options(usize),
}

impl GameValue {
    pub fn nimber(value: u64) -> Self {
        Self(GameValueNode::NumberNimber(0.into(), value))
    }

    pub fn as_number(&self) -> Option<Dyadic<i64>> {
        match self.0 {
            GameValueNode::NumberNimber(n, 0) => Some(n),
            _ => None,
        }
    }

    pub fn as_nimber(&self) -> Option<u64> {
        match self.0 {
            GameValueNode::NumberNimber(q, n) if q.numerator() == 0 => Some(n),
            _ => None,
        }
    }

    /// Returns `(q, n)` when the value is `q + *n`.
    pub fn as_number_nimber(&self) -> Option<(Dyadic<i64>, u64)> {
        match self.0 {
            GameValueNode::NumberNimber(q, n) => Some((q, n)),
            _ => None,
        }
    }

    pub fn outcome(&self) -> PartisanOutcome {
        match self.partial_cmp(&Self::from(0)) {
            Some(Ordering::Greater) => PartisanOutcome::Left,
            Some(Ordering::Less) => PartisanOutcome::Right,
            Some(Ordering::Equal) => PartisanOutcome::Previous,
            None => PartisanOutcome::Next,
        }
    }

    /// Whether the selected starting player can force a win.
    pub fn wins(&self, player: Player) -> bool {
        match player {
            Player::Left => !self.leq(&Self::from(0)),
            Player::Right => !Self::from(0).leq(self),
        }
    }

    /// Removes dominated and reversible options.
    pub fn from_options(
        left: impl IntoIterator<Item = Self>,
        right: impl IntoIterator<Item = Self>,
    ) -> Self {
        let mut left: Vec<_> = left.into_iter().collect();
        let mut right: Vec<_> = right.into_iter().collect();
        loop {
            if left.iter().chain(&right).all(|g| g.as_number().is_some()) {
                let l = left.iter().filter_map(Self::as_number).max();
                let r = right.iter().filter_map(Self::as_number).min();
                if let Some(n) = Dyadic::simplest_between(l, r) {
                    return n.into();
                }
                let (l, r) = (l.unwrap(), r.unwrap());
                if l == r {
                    return Self(GameValueNode::NumberNimber(l, 1));
                }
                return Self(GameValueNode::Options(Rc::new((
                    vec![l.into()],
                    vec![r.into()],
                ))));
            }
            if !right.is_empty()
                && let Some((number, _)) = left.first().and_then(Self::as_number_nimber)
            {
                let nimber = |g: &Self| {
                    g.as_number_nimber()
                        .filter(|&(q, _)| q == number)
                        .map(|(_, n)| n)
                };
                let l: Option<Vec<_>> = left.iter().map(nimber).collect();
                let r: Option<Vec<_>> = right.iter().map(nimber).collect();
                if let (Some(l), Some(r)) = (l, r) {
                    let value = game_mex(l);
                    if value == game_mex(r) {
                        return Self(GameValueNode::NumberNimber(number, value));
                    }
                }
            }
            let count = left.len() + right.len();
            Self::remove_dominated(&mut left, Player::Left);
            Self::remove_dominated(&mut right, Player::Right);
            if count != left.len() + right.len() {
                continue;
            }
            let options = Rc::new((left, right));
            let game = Self(GameValueNode::Options(options.clone()));
            let mut cache = HashMap::new();
            // Bypasses preserve the game, so all checks use the same candidate.
            let mut reversible = Vec::new();
            for player in [Player::Left, Player::Right] {
                let moves = match player {
                    Player::Left => &options.0,
                    Player::Right => &options.1,
                };
                for (i, g) in moves.iter().enumerate() {
                    if let Some((q, n)) = g.as_number_nimber()
                        && n > 1
                    {
                        let number = Self::from(q);
                        let (lower, upper) = match player {
                            Player::Left => (&game, &number),
                            Player::Right => (&number, &game),
                        };
                        // A strict inequality against q excludes every reply q + *m.
                        if lower.leq_memoized(upper, &mut cache)
                            && !upper.leq_memoized(lower, &mut cache)
                        {
                            continue;
                        }
                    }
                    if let Some(reply) = g.options(player.opponent()).find(|r| match player {
                        Player::Left => r.leq_memoized(&game, &mut cache),
                        Player::Right => game.leq_memoized(r, &mut cache),
                    }) {
                        reversible.push((player, i, reply));
                    }
                }
            }
            if reversible.is_empty() {
                return game;
            }
            drop(game);
            (left, right) = Rc::try_unwrap(options).unwrap();
            for (player, i, reply) in reversible.into_iter().rev() {
                let moves = if player == Player::Left {
                    &mut left
                } else {
                    &mut right
                };
                moves.remove(i);
                moves.extend(reply.options(player));
            }
        }
    }

    pub fn options(&self, player: Player) -> impl Iterator<Item = Self> + '_ {
        let (single, nimbers, general) = match &self.0 {
            GameValueNode::NumberNimber(n, 0) => {
                let option = if n.exponent() == 0 {
                    match player {
                        Player::Left if n.numerator() > 0 => Some(Dyadic::from(n.numerator() - 1)),
                        Player::Right if n.numerator() < 0 => Some(Dyadic::from(n.numerator() + 1)),
                        _ => None,
                    }
                } else {
                    let numerator = (n.numerator() >> 1) + i64::from(player == Player::Right);
                    Some(Dyadic::new(numerator, n.exponent() - 1))
                };
                (option, None, &[][..])
            }
            GameValueNode::NumberNimber(q, n) => (None, Some((*q, *n)), &[][..]),
            GameValueNode::Options(options) => {
                let (left, right) = &**options;
                let moves = if player == Player::Left { left } else { right };
                (None, None, moves.as_slice())
            }
        };
        single
            .into_iter()
            .map(Self::from)
            .chain(
                nimbers.into_iter().flat_map(|(q, n)| {
                    (0..n).map(move |m| Self(GameValueNode::NumberNimber(q, m)))
                }),
            )
            .chain(general.iter().cloned())
    }

    fn leq(&self, rhs: &Self) -> bool {
        self.leq_memoized(rhs, &mut HashMap::new())
    }

    fn leq_memoized(
        &self,
        rhs: &Self,
        cache: &mut HashMap<(GameValueKey, GameValueKey), bool>,
    ) -> bool {
        match (&self.0, &rhs.0) {
            (GameValueNode::NumberNimber(a, n), GameValueNode::NumberNimber(b, m)) => {
                return a < b || a == b && n == m;
            }
            (GameValueNode::Options(a), GameValueNode::Options(b)) if Rc::ptr_eq(a, b) => {
                return true;
            }
            _ => {}
        }
        let key = (self.key(), rhs.key());
        if let Some(&value) = cache.get(&key) {
            return value;
        }
        let value = if let GameValueNode::NumberNimber(q, n) = self.0
            && n != 0
        {
            let number = Self::from(q);
            // Try the explicit options before the nimber's range.
            !rhs.options(Player::Right)
                .any(|g| g.leq_memoized(self, cache))
                && !rhs.leq_memoized(&number, cache)
                && (number.leq_memoized(rhs, cache)
                    || !self
                        .options(Player::Left)
                        .skip(1)
                        .any(|g| rhs.leq_memoized(&g, cache)))
        } else {
            !self
                .options(Player::Left)
                .any(|g| rhs.leq_memoized(&g, cache))
                && if let GameValueNode::NumberNimber(q, n) = rhs.0
                    && n != 0
                {
                    let number = Self::from(q);
                    !number.leq_memoized(self, cache)
                        && (self.leq_memoized(&number, cache)
                            || !rhs
                                .options(Player::Right)
                                .skip(1)
                                .any(|g| g.leq_memoized(self, cache)))
                } else {
                    !rhs.options(Player::Right)
                        .any(|g| g.leq_memoized(self, cache))
                }
        };
        cache.insert(key, value);
        value
    }

    fn negated(&self, cache: &mut HashMap<usize, Self>) -> Self {
        match &self.0 {
            GameValueNode::NumberNimber(q, n) => Self(GameValueNode::NumberNimber(-*q, *n)),
            GameValueNode::Options(options) => {
                let key = Rc::as_ptr(options) as usize;
                if let Some(value) = cache.get(&key) {
                    return value.clone();
                }
                let (left, right) = &**options;
                let value = Self(GameValueNode::Options(Rc::new((
                    right.iter().map(|g| g.negated(cache)).collect(),
                    left.iter().map(|g| g.negated(cache)).collect(),
                ))));
                cache.insert(key, value.clone());
                value
            }
        }
    }

    fn sum(&self, rhs: &Self, cache: &mut HashMap<(GameValueKey, GameValueKey), Self>) -> Self {
        if let (Some((a, n)), Some((b, m))) = (self.as_number_nimber(), rhs.as_number_nimber()) {
            return Self(GameValueNode::NumberNimber(a + b, n ^ m));
        }
        if self.as_nimber() == Some(0) {
            return rhs.clone();
        }
        if rhs.as_nimber() == Some(0) {
            return self.clone();
        }
        let mut key = (self.key(), rhs.key());
        if key.0 > key.1 {
            std::mem::swap(&mut key.0, &mut key.1);
        }
        if let Some(value) = cache.get(&key) {
            return value.clone();
        }
        // Strong number avoidance: only the non-number component needs moves.
        let mut options = |player| {
            let mut moves = Vec::new();
            if self.as_number().is_none() {
                for g in self.options(player) {
                    moves.push(g.sum(rhs, cache));
                }
            }
            if rhs.as_number().is_none() {
                for g in rhs.options(player) {
                    moves.push(self.sum(&g, cache));
                }
            }
            moves
        };
        let value = Self::from_options(options(Player::Left), options(Player::Right));
        cache.insert(key, value.clone());
        value
    }

    // Pointer keys are local to an operation whose operands keep every input node alive.
    fn key(&self) -> GameValueKey {
        match &self.0 {
            GameValueNode::NumberNimber(q, n) => GameValueKey::NumberNimber(*q, *n),
            GameValueNode::Options(options) => GameValueKey::Options(Rc::as_ptr(options) as usize),
        }
    }

    fn remove_dominated(options: &mut Vec<Self>, player: Player) {
        let mut cache = HashMap::new();
        let mut retained = vec![true; options.len()];
        for i in 0..options.len() {
            let dominated = (0..options.len()).any(|j| {
                i != j
                    && retained[j]
                    && match player {
                        Player::Left => options[i].leq_memoized(&options[j], &mut cache),
                        Player::Right => options[j].leq_memoized(&options[i], &mut cache),
                    }
            });
            retained[i] = !dominated;
        }
        let mut retained = retained.into_iter();
        options.retain(|_| retained.next().unwrap());
    }
}

impl From<i64> for GameValue {
    fn from(value: i64) -> Self {
        Dyadic::from(value).into()
    }
}
impl From<Dyadic<i64>> for GameValue {
    fn from(value: Dyadic<i64>) -> Self {
        Self(GameValueNode::NumberNimber(value, 0))
    }
}
impl Neg for GameValue {
    type Output = Self;
    fn neg(self) -> Self {
        self.negated(&mut HashMap::new())
    }
}
impl Add for GameValue {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        self.sum(&rhs, &mut HashMap::new())
    }
}
impl Sub for GameValue {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        if self.key() == rhs.key() {
            Self::from(0)
        } else {
            self + -rhs
        }
    }
}
impl PartialEq for GameValue {
    fn eq(&self, rhs: &Self) -> bool {
        let mut cache = HashMap::new();
        self.leq_memoized(rhs, &mut cache) && rhs.leq_memoized(self, &mut cache)
    }
}
impl Eq for GameValue {}
impl PartialOrd for GameValue {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        let mut cache = HashMap::new();
        match (
            self.leq_memoized(rhs, &mut cache),
            rhs.leq_memoized(self, &mut cache),
        ) {
            (true, true) => Some(Ordering::Equal),
            (true, false) => Some(Ordering::Less),
            (false, true) => Some(Ordering::Greater),
            (false, false) => None,
        }
    }
}
impl fmt::Display for GameValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            GameValueNode::NumberNimber(q, 0) => write!(f, "{q}"),
            GameValueNode::NumberNimber(q, n) => {
                if q.numerator() != 0 {
                    write!(f, "{q}+")?;
                }
                if *n == 1 {
                    f.write_str("*")
                } else {
                    write!(f, "*{n}")
                }
            }
            GameValueNode::Options(options) => {
                let (left, right) = &**options;
                if left.len() == 1 && right.len() == 1 {
                    match (left[0].as_nimber(), right[0].as_nimber()) {
                        (Some(0), Some(1)) => return f.write_str("↑"),
                        (Some(1), Some(0)) => return f.write_str("↓"),
                        _ => {}
                    }
                }
                f.write_str("{")?;
                for (separator, options) in [("", left), ("|", right)] {
                    f.write_str(separator)?;
                    for (i, g) in options.iter().enumerate() {
                        if i > 0 {
                            f.write_str(",")?;
                        }
                        write!(f, "{g}")?;
                    }
                }
                f.write_str("}")
            }
        }
    }
}
impl fmt::Debug for GameValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{Xorshift, testutil::integer_boundary_values};

    #[test]
    fn number_nimbers() {
        let mut rng = Xorshift::default();
        let nimbers: Vec<_> = integer_boundary_values!(u64)
            .into_iter()
            .chain(rng.random_iter(..).take(1024))
            .collect();
        for n in nimbers {
            let random = Dyadic::new(rng.random(-8192..=8192), rng.random(0..=8));
            for q in [
                0.into(),
                (-8192).into(),
                8192.into(),
                Dyadic::new(-1, 1),
                Dyadic::new(1, 1),
                random,
            ] {
                let value = GameValue::from(q) + GameValue::nimber(n);
                assert_eq!(
                    GameValue::from_options([q.into()], [q.into()]).as_number_nimber(),
                    Some((q, 1))
                );
                assert_eq!(value.as_number_nimber(), Some((q, n)));
                assert_eq!(value.as_number(), (n == 0).then_some(q));
                assert_eq!(value.as_nimber(), (q.numerator() == 0).then_some(n));
                let prefix = if n != 0 && q.numerator() != 0 {
                    format!("{q}+")
                } else {
                    String::new()
                };
                let suffix = match n {
                    0 => q.to_string(),
                    1 => "*".to_owned(),
                    _ => format!("*{n}"),
                };
                assert_eq!(value.to_string(), prefix + &suffix);
                assert_eq!((-value.clone()).as_number_nimber(), Some((-q, n)));
                assert_eq!(value.clone() - value.clone(), 0.into());
                let m = rng.rand64();
                let r = Dyadic::new(rng.random(-8192..=8192), rng.random(0..=8));
                let other = GameValue::from(r) + GameValue::nimber(m);
                assert_eq!(
                    (value.clone() + other.clone()).as_number_nimber(),
                    Some((q + r, n ^ m))
                );
                assert_eq!(
                    (value.clone() - other).as_number_nimber(),
                    Some((q - r, n ^ m))
                );
                if n != 0 {
                    for player in [Player::Left, Player::Right] {
                        assert!(
                            value
                                .options(player)
                                .take(3)
                                .enumerate()
                                .all(|(m, g)| { g.as_number_nimber() == Some((q, m as u64)) })
                        );
                        assert_eq!(value.options(player).take(3).count(), n.min(3) as usize);
                    }
                }
            }
            if n != 0 {
                let star = GameValue::nimber(n);
                let switch = GameValue::from_options([1.into()], [(-1).into()]);
                assert_eq!(star.partial_cmp(&switch), None);
                assert_eq!(switch.partial_cmp(&star), None);
                let up = GameValue::from_options([0.into()], [star]);
                assert!(up > GameValue::from(0));
                assert_eq!(up.clone() - up, 0.into());
            }
        }
    }

    #[test]
    fn order_and_sum_against_play() {
        let mut rng = Xorshift::default();
        let mut games = vec![
            (vec![], vec![]),
            (vec![0], vec![]),
            (vec![], vec![0]),
            (vec![0], vec![0]),
        ];
        // All birthday-two option sets, followed by sampled larger DAGs.
        for l in 0..16 {
            for r in 0..16 {
                games.push((
                    (0..4).filter(|&i| l >> i & 1 != 0).collect(),
                    (0..4).filter(|&i| r >> i & 1 != 0).collect(),
                ));
            }
        }
        for n in games.len()..games.len() + 32 {
            games.push((
                (0..3).map(|_| rng.random(0..n)).collect(),
                (0..3).map(|_| rng.random(0..n)).collect(),
            ));
        }
        // Integers in [-2, 2], then their midpoints, each with a three-bit Nim heap.
        let mut numbers = vec![(0i64, Vec::new(), Vec::new())];
        let mut indices = HashMap::from([(0, 0)]);
        for step in [2, 1] {
            for q in (step..=4).step_by(step as usize).flat_map(|q| [-q, q]) {
                if indices.contains_key(&q) {
                    continue;
                }
                let left = (step == 1 || q > 0)
                    .then(|| indices[&(q - step)])
                    .into_iter()
                    .collect();
                let right = (step == 1 || q < 0)
                    .then(|| indices[&(q + step)])
                    .into_iter()
                    .collect();
                indices.insert(q, numbers.len());
                numbers.push((q, left, right));
            }
        }
        let start = games.len();
        let mut compact = Vec::new();
        for (i, &(q, ref left, ref right)) in numbers.iter().enumerate() {
            for n in 0..8 {
                games.push((
                    left.iter()
                        .map(|j| start + j * 8 + n)
                        .chain((0..n).map(|m| start + i * 8 + m))
                        .collect(),
                    right
                        .iter()
                        .map(|j| start + j * 8 + n)
                        .chain((0..n).map(|m| start + i * 8 + m))
                        .collect(),
                ));
                compact.push((games.len() - 1, (Dyadic::new(q, 1), n as u64)));
            }
        }
        let symmetric = games.len();
        for (i, &(q, _, _)) in numbers.iter().enumerate() {
            for mask in 1..16 {
                let left: Vec<_> = (0..4)
                    .filter(|n| mask >> n & 1 != 0)
                    .flat_map(|n| [start + i * 8 + n; 2])
                    .collect();
                games.push((left.clone(), left.into_iter().rev().collect()));
                let mex = (0..).find(|n| mask >> n & 1 == 0).unwrap();
                compact.push((games.len() - 1, (Dyadic::new(q, 1), mex)));
            }
        }
        let asymmetric = games.len();
        for i in 0..numbers.len() {
            for n in 0..8 {
                for m in 0..8 {
                    games.push((vec![start + i * 8 + n], vec![start + i * 8 + m]));
                }
            }
        }
        let mut values: Vec<GameValue> = Vec::new();
        for (left, right) in &games {
            values.push(GameValue::from_options(
                left.iter().map(|&i| values[i].clone()),
                right.iter().map(|&i| values[i].clone()),
            ));
        }
        for (i, expected) in compact {
            assert_eq!(values[i].as_number_nimber(), Some(expected));
        }
        let zero = GameValue::from(0);
        let mut differences = HashMap::new();
        let mut sums = HashMap::new();
        for (i, g) in values.iter().enumerate() {
            let winning = [false, true].map(|right| wins(&games, i, 0, right, false, &mut sums));
            for (player, win) in [Player::Left, Player::Right].into_iter().zip(winning) {
                assert_eq!(g.wins(player), win);
                let options: Vec<_> = g.options(player).collect();
                for (j, option) in options.iter().enumerate() {
                    for other in options.iter().skip(j + 1) {
                        assert_eq!(option.partial_cmp(other), None, "dominated options in {g}");
                    }
                    for reply in option.options(player.opponent()) {
                        let order = reply.partial_cmp(g);
                        assert_ne!(order, Some(Ordering::Equal), "reversible option in {g}");
                        assert_ne!(
                            order,
                            Some(if player == Player::Left {
                                Ordering::Less
                            } else {
                                Ordering::Greater
                            }),
                            "reversible option in {g}"
                        );
                    }
                }
            }
            let outcome = match winning {
                [true, true] => PartisanOutcome::Next,
                [false, false] => PartisanOutcome::Previous,
                [true, false] => PartisanOutcome::Left,
                [false, true] => PartisanOutcome::Right,
            };
            assert_eq!(g.outcome(), outcome);
            assert_eq!(g.clone() - g.clone(), zero, "G-G for {i}: {g}");
            assert_eq!(g.clone() + -g.clone(), zero);
            assert_eq!(-(-g.clone()), *g);
            let partners: Vec<_> = if i < 260 {
                (0..260).collect()
            } else if i >= asymmetric {
                (start..symmetric).collect()
            } else if i >= start {
                (start..asymmetric).collect()
            } else {
                (0..32).map(|_| rng.random(0..values.len())).collect()
            };
            for j in partners {
                let expected = match (
                    wins(&games, i, j, false, true, &mut differences),
                    wins(&games, i, j, true, true, &mut differences),
                ) {
                    (false, false) => Some(Ordering::Equal),
                    (true, false) => Some(Ordering::Greater),
                    (false, true) => Some(Ordering::Less),
                    (true, true) => None,
                };
                assert_eq!(
                    g.partial_cmp(&values[j]),
                    expected,
                    "{i} vs {j}: {g}, {}",
                    values[j]
                );
                assert_eq!(g == &values[j], expected == Some(Ordering::Equal));
                if (i < 260 && j >= 4) || i >= asymmetric {
                    continue;
                }
                assert_eq!((g.clone() - values[j].clone()).partial_cmp(&zero), expected);
                let sum = g.clone() + values[j].clone();
                assert_eq!(sum, values[j].clone() + g.clone());
                for player in [Player::Left, Player::Right] {
                    assert_eq!(
                        sum.wins(player),
                        wins(&games, i, j, player == Player::Right, false, &mut sums),
                        "{i} + {j}, {player:?}"
                    );
                }
            }
        }
    }

    // Direct minimax on raw trees, independently of canonicalization and XOR.
    fn wins(
        games: &[(Vec<usize>, Vec<usize>)],
        a: usize,
        b: usize,
        right: bool,
        subtract: bool,
        cache: &mut HashMap<(usize, usize, bool), bool>,
    ) -> bool {
        if let Some(&result) = cache.get(&(a, b, right)) {
            return result;
        }
        let first = if right { &games[a].1 } else { &games[a].0 };
        let second = if right ^ subtract {
            &games[b].1
        } else {
            &games[b].0
        };
        let result = first
            .iter()
            .any(|&c| !wins(games, c, b, !right, subtract, cache))
            || second
                .iter()
                .any(|&c| !wins(games, a, c, !right, subtract, cache));
        cache.insert((a, b, right), result);
        result
    }
}
