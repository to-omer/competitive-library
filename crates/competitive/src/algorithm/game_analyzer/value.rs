use super::{Dyadic, PartisanOutcome, Player, game_mex};
use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt,
    hash::{DefaultHasher, Hash, Hasher},
    iter::Sum,
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
    NumberUpNimber(Dyadic<i64>, i64, u64),
    Switches(Rc<GameValueSwitches>),
    // A numeric translation shares the canonical options of a non-number.
    Options(Dyadic<i64>, Rc<(Vec<GameValue>, Vec<GameValue>)>),
}

// Memo keys retain general nodes and compare compact values by their contents.
impl PartialEq for GameValueNode {
    fn eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::NumberUpNimber(a, k, n), Self::NumberUpNimber(b, l, m)) => {
                a == b && k == l && n == m
            }
            (Self::Switches(a), Self::Switches(b)) => {
                Rc::ptr_eq(a, b)
                    || a.number == b.number
                        && a.up == b.up
                        && a.nimber == b.nimber
                        && a.temperatures[a.start..] == b.temperatures[b.start..]
            }
            (Self::Options(q, a), Self::Options(r, b)) => q == r && Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}
impl Eq for GameValueNode {}
impl Hash for GameValueNode {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        match self {
            Self::NumberUpNimber(q, k, n) => (q, k, n).hash(state),
            Self::Switches(v) => (v.number, v.up, v.nimber, v.fingerprint).hash(state),
            Self::Options(q, v) => {
                Rc::as_ptr(v).hash(state);
                if q.numerator() != 0 {
                    q.hash(state);
                }
            }
        }
    }
}

#[derive(Clone)]
struct GameValueSwitches {
    number: Dyadic<i64>,
    up: i64,
    nimber: u64,
    temperatures: Rc<Vec<Dyadic<i64>>>,
    start: usize,
    // XOR of the remaining temperatures' hashes; equality still checks the contents.
    fingerprint: u64,
}

impl GameValue {
    pub fn nimber(value: u64) -> Self {
        Self::number_up_nimber(0.into(), 0, value)
    }

    /// Constructs `number + up * ↑ + *nimber` without expanding options.
    pub fn number_up_nimber(number: Dyadic<i64>, up: i64, nimber: u64) -> Self {
        Self(GameValueNode::NumberUpNimber(number, up, nimber))
    }

    pub fn as_number(&self) -> Option<Dyadic<i64>> {
        match self.0 {
            GameValueNode::NumberUpNimber(n, 0, 0) => Some(n),
            _ => None,
        }
    }

    pub fn as_nimber(&self) -> Option<u64> {
        match self.0 {
            GameValueNode::NumberUpNimber(q, 0, n) if q.numerator() == 0 => Some(n),
            _ => None,
        }
    }

    /// Returns `(q, n)` when the value is `q + *n`.
    pub fn as_number_nimber(&self) -> Option<(Dyadic<i64>, u64)> {
        match self.0 {
            GameValueNode::NumberUpNimber(q, 0, n) => Some((q, n)),
            _ => None,
        }
    }

    /// Returns `(q, k, n)` when the value is `q + k↑ + *n`.
    pub fn as_number_up_nimber(&self) -> Option<(Dyadic<i64>, i64, u64)> {
        match self.0 {
            GameValueNode::NumberUpNimber(q, k, n) => Some((q, k, n)),
            _ => None,
        }
    }

    /// Returns `(q, k, n, temperatures)` for `q + k↑ + *n + Σ{t|-t}`.
    /// Temperatures are positive and strictly decreasing; the slice may be empty.
    #[allow(clippy::type_complexity)] // Borrows the components without a separate public type.
    pub fn as_switch_sum(&self) -> Option<(Dyadic<i64>, i64, u64, &[Dyadic<i64>])> {
        match &self.0 {
            GameValueNode::NumberUpNimber(q, k, n) => Some((*q, *k, *n, &[])),
            GameValueNode::Switches(value) => Some((
                value.number,
                value.up,
                value.nimber,
                &value.temperatures[value.start..],
            )),
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
        Self::from_options_memoized(
            left.into_iter().collect(),
            right.into_iter().collect(),
            &mut HashMap::new(),
        )
    }

    pub fn options(&self, player: Player) -> impl Iterator<Item = Self> + '_ {
        let mut offset = Dyadic::from(0);
        let (single, nimbers, general) = match &self.0 {
            GameValueNode::NumberUpNimber(n, 0, 0) => {
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
                (option.map(Self::from), None, &[][..])
            }
            GameValueNode::NumberUpNimber(q, 0, n) => (None, Some((*q, *n)), &[][..]),
            GameValueNode::NumberUpNimber(q, k, 1)
                if matches!((*k, player), (1, Player::Left) | (-1, Player::Right)) =>
            {
                (None, Some((*q, 2)), &[][..])
            }
            GameValueNode::NumberUpNimber(q, k, n) => {
                let sign = if *k > 0 { 1 } else { -1 };
                let moves_to_number = (*k > 0) == (player == Player::Left);
                let option = if moves_to_number {
                    Self::from(*q)
                } else {
                    Self::number_up_nimber(*q, k - sign, n ^ 1)
                };
                (Some(option), None, &[][..])
            }
            GameValueNode::Switches(value) => {
                let temperatures = &value.temperatures[value.start..];
                let t = if player == Player::Left {
                    temperatures[0]
                } else {
                    -temperatures[0]
                };
                let option = if temperatures.len() == 1 {
                    Self::number_up_nimber(value.number + t, value.up, value.nimber)
                } else {
                    Self(GameValueNode::Switches(Rc::new(GameValueSwitches {
                        number: value.number + t,
                        start: value.start + 1,
                        fingerprint: value.fingerprint ^ Self::temperature_hash(temperatures[0]),
                        ..(**value).clone()
                    })))
                };
                (Some(option), None, &[][..])
            }
            GameValueNode::Options(q, options) => {
                offset = *q;
                let (left, right) = &**options;
                let moves = if player == Player::Left { left } else { right };
                (None, None, moves.as_slice())
            }
        };
        single
            .into_iter()
            .chain(
                nimbers
                    .into_iter()
                    .flat_map(|(q, n)| (0..n).map(move |m| Self::number_up_nimber(q, 0, m))),
            )
            .chain(general.iter().map(move |g| g.translated(offset)))
    }

    fn from_options_memoized(
        mut left: Vec<Self>,
        mut right: Vec<Self>,
        cache: &mut HashMap<GameValueNode, HashMap<GameValueNode, bool>>,
    ) -> Self {
        loop {
            if left.iter().chain(&right).all(|g| g.as_number().is_some()) {
                let l = left.iter().filter_map(Self::as_number).max();
                let r = right.iter().filter_map(Self::as_number).min();
                if let Some(n) = Dyadic::simplest_between(l, r) {
                    return n.into();
                }
                let (l, r) = (l.unwrap(), r.unwrap());
                if l == r {
                    return Self::number_up_nimber(l, 0, 1);
                }
                let (sum, difference) = (l + r, l - r);
                let temperature = Dyadic::new(difference.numerator(), difference.exponent() + 1);
                return Self::from_switches(
                    Dyadic::new(sum.numerator(), sum.exponent() + 1),
                    0,
                    0,
                    vec![temperature],
                    Self::temperature_hash(temperature),
                );
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
                        return Self::number_up_nimber(number, 0, value);
                    }
                }
            }
            for (moves, replies, sign) in [(&left, &right, 1), (&right, &left, -1)] {
                if replies.len() != 1 {
                    continue;
                }
                if let Some((q, k, n)) = replies[0].as_number_up_nimber() {
                    if moves.len() == 1
                        && moves[0].as_number() == Some(q)
                        && (k == 0 || k.signum() == sign)
                    {
                        return Self::number_up_nimber(q, k + sign, n ^ 1);
                    }
                    if k == 0 && n == 0 && moves.len() == 2 {
                        let a = moves[0].as_number_nimber();
                        let b = moves[1].as_number_nimber();
                        if (a == Some((q, 0)) && b == Some((q, 1)))
                            || (a == Some((q, 1)) && b == Some((q, 0)))
                        {
                            return Self::number_up_nimber(q, sign, 1);
                        }
                    }
                }
            }
            if left.len() == 1
                && right.len() == 1
                && let (Some((a, k, n, ts)), Some((b, l, m, us))) =
                    (left[0].as_switch_sum(), right[0].as_switch_sum())
                && k == l
                && n == m
                && ts == us
                && a > b
            {
                let (sum, difference) = (a + b, a - b);
                let temperature = Dyadic::new(difference.numerator(), difference.exponent() + 1);
                if ts.first().is_none_or(|&t| temperature > t) {
                    return Self::from_switches(
                        Dyadic::new(sum.numerator(), sum.exponent() + 1),
                        k,
                        n,
                        std::iter::once(temperature)
                            .chain(ts.iter().copied())
                            .collect(),
                        Self::temperature_hash(temperature) ^ left[0].switch_hash(),
                    );
                }
            }
            let count = left.len() + right.len();
            Self::remove_dominated(&mut left, Player::Left, cache);
            Self::remove_dominated(&mut right, Player::Right, cache);
            if count != left.len() + right.len() {
                continue;
            }
            let options = Rc::new((left, right));
            let game = Self(GameValueNode::Options(0.into(), options.clone()));
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
                        if lower.leq_memoized(upper, cache) && !upper.leq_memoized(lower, cache) {
                            continue;
                        }
                    }
                    if let Some(reply) = g.options(player.opponent()).find(|r| match player {
                        Player::Left => r.leq_memoized(&game, cache),
                        Player::Right => game.leq_memoized(r, cache),
                    }) {
                        reversible.push((player, i, reply));
                    }
                }
            }
            if reversible.is_empty() {
                return game;
            }
            drop(game);
            (left, right) = Rc::unwrap_or_clone(options);
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

    fn leq(&self, rhs: &Self) -> bool {
        self.leq_memoized(rhs, &mut HashMap::new())
    }

    fn leq_memoized(
        &self,
        rhs: &Self,
        cache: &mut HashMap<GameValueNode, HashMap<GameValueNode, bool>>,
    ) -> bool {
        if let (Some((a, k, n, ts)), Some((b, l, m, us))) =
            (self.as_switch_sum(), rhs.as_switch_sum())
        {
            // Both players take the hottest remaining switch. At zero, the tail
            // decides the winner with the turn reversed once per switch.
            let mut number = a;
            let mut odd = false;
            for t in Self::switch_difference(ts, us) {
                // The unplayed alternating tail has the sign of the next term.
                if !odd && number >= b || odd && number <= b {
                    return odd;
                }
                number = if odd { number - t } else { number + t };
                odd = !odd;
            }
            return match number.cmp(&b) {
                Ordering::Less => true,
                Ordering::Greater => false,
                Ordering::Equal => {
                    let up = if odd { l - k } else { k - l };
                    let nimber = n ^ m;
                    let wins = up > 0 || up == 0 && nimber != 0 || up == -1 && nimber == 1;
                    wins == odd
                }
            };
        }
        if let (GameValueNode::Options(a, g), GameValueNode::Options(b, h)) = (&self.0, &rhs.0)
            && Rc::ptr_eq(g, h)
        {
            return a <= b;
        }
        // Normalize only memo keys: from_options also compares unreduced candidates.
        let offset = -self.number_offset();
        let left = self.translated(offset).0;
        let right = rhs.translated(offset).0;
        if let Some(&value) = cache.get(&left).and_then(|row| row.get(&right)) {
            return value;
        }
        // Try explicit options before expanding a compact value.
        let mut sides = [(self, rhs, Player::Left), (rhs, self, Player::Right)];
        if self.as_switch_sum().is_some() {
            sides.swap(0, 1);
        }
        let value = sides.into_iter().all(|(game, other, player)| {
            let mut preferred = |a: &Self, b: &Self| match player {
                Player::Left => b.leq_memoized(a, cache),
                Player::Right => a.leq_memoized(b, cache),
            };
            if let GameValueNode::NumberUpNimber(q, 0, n) = game.0
                && n != 0
            {
                let number = Self::from(q);
                !preferred(&number, other)
                    && (preferred(other, &number)
                        || !game.options(player).skip(1).any(|g| preferred(&g, other)))
            } else {
                !game.options(player).any(|g| preferred(&g, other))
            }
        });
        cache.entry(left).or_default().insert(right, value);
        value
    }

    fn negated(&self, cache: &mut HashMap<usize, Self>) -> Self {
        match &self.0 {
            GameValueNode::NumberUpNimber(q, k, n) => Self::number_up_nimber(-*q, -*k, *n),
            GameValueNode::Switches(value) => {
                Self(GameValueNode::Switches(Rc::new(GameValueSwitches {
                    number: -value.number,
                    up: -value.up,
                    ..(**value).clone()
                })))
            }
            GameValueNode::Options(q, options) => {
                let key = Rc::as_ptr(options) as usize;
                if let Some(value) = cache.get(&key) {
                    return value.translated(-*q);
                }
                let (left, right) = &**options;
                let value = Self(GameValueNode::Options(
                    0.into(),
                    Rc::new((
                        right.iter().map(|g| g.negated(cache)).collect(),
                        left.iter().map(|g| g.negated(cache)).collect(),
                    )),
                ));
                cache.insert(key, value.clone());
                value.translated(-*q)
            }
        }
    }

    fn sum(
        &self,
        rhs: &Self,
        cache: &mut HashMap<(GameValueNode, GameValueNode), Self>,
        comparisons: &mut HashMap<GameValueNode, HashMap<GameValueNode, bool>>,
    ) -> Self {
        if let (GameValueNode::NumberUpNimber(a, k, n), GameValueNode::NumberUpNimber(b, l, m)) =
            (&self.0, &rhs.0)
        {
            return Self::number_up_nimber(*a + *b, k + l, n ^ m);
        }
        if let Some(q) = self.as_number() {
            return rhs.translated(q);
        }
        if let Some(q) = rhs.as_number() {
            return self.translated(q);
        }
        if let (Some((a, k, n, ts)), Some((b, l, m, us))) =
            (self.as_switch_sum(), rhs.as_switch_sum())
        {
            return Self::from_switches(
                a + b,
                k + l,
                n ^ m,
                Self::switch_difference(ts, us).collect(),
                self.switch_hash() ^ rhs.switch_hash(),
            );
        }
        // Cache the sum of the untranslated parts, sharing it across numeric offsets.
        let (a, b) = (self.number_offset(), rhs.number_offset());
        if a.numerator() != 0 || b.numerator() != 0 {
            return self
                .translated(-a)
                .sum(&rhs.translated(-b), cache, comparisons)
                .translated(a + b);
        }
        let mut key = (self.0.clone(), rhs.0.clone());
        let swap = match (&key.0, &key.1) {
            (GameValueNode::Options(_, a), GameValueNode::Options(_, b)) => {
                Rc::as_ptr(a) > Rc::as_ptr(b)
            }
            (GameValueNode::Options(..), _) => true,
            _ => false,
        };
        if swap {
            std::mem::swap(&mut key.0, &mut key.1);
        }
        if let Some(value) = cache.get(&key) {
            return value.clone();
        }
        let mut options = |player| {
            let mut moves = Vec::new();
            for g in self.options(player) {
                moves.push(g.sum(rhs, cache, comparisons));
            }
            for g in rhs.options(player) {
                moves.push(self.sum(&g, cache, comparisons));
            }
            moves
        };
        let value =
            Self::from_options_memoized(options(Player::Left), options(Player::Right), comparisons);
        cache.insert(key, value.clone());
        value
    }

    fn number_offset(&self) -> Dyadic<i64> {
        match &self.0 {
            GameValueNode::NumberUpNimber(q, ..) | GameValueNode::Options(q, _) => *q,
            GameValueNode::Switches(v) => v.number,
        }
    }

    fn translated(&self, number: Dyadic<i64>) -> Self {
        if number.numerator() == 0 {
            return self.clone();
        }
        match &self.0 {
            GameValueNode::NumberUpNimber(q, k, n) => Self::number_up_nimber(*q + number, *k, *n),
            GameValueNode::Switches(v) => {
                Self(GameValueNode::Switches(Rc::new(GameValueSwitches {
                    number: v.number + number,
                    ..(**v).clone()
                })))
            }
            GameValueNode::Options(q, options) => {
                Self(GameValueNode::Options(*q + number, options.clone()))
            }
        }
    }

    fn temperature_hash(t: Dyadic<i64>) -> u64 {
        let mut hasher = DefaultHasher::new();
        t.hash(&mut hasher);
        hasher.finish()
    }

    fn switch_hash(&self) -> u64 {
        match &self.0 {
            GameValueNode::NumberUpNimber(..) => 0,
            GameValueNode::Switches(v) => v.fingerprint,
            GameValueNode::Options(..) => unreachable!(),
        }
    }

    fn from_switches(
        q: Dyadic<i64>,
        k: i64,
        n: u64,
        temperatures: Vec<Dyadic<i64>>,
        fingerprint: u64,
    ) -> Self {
        if temperatures.is_empty() {
            Self::number_up_nimber(q, k, n)
        } else {
            Self(GameValueNode::Switches(Rc::new(GameValueSwitches {
                number: q,
                up: k,
                nimber: n,
                fingerprint,
                temperatures: Rc::new(temperatures),
                start: 0,
            })))
        }
    }

    fn switch_difference<'a>(
        mut left: &'a [Dyadic<i64>],
        mut right: &'a [Dyadic<i64>],
    ) -> impl Iterator<Item = Dyadic<i64>> + 'a {
        std::iter::from_fn(move || {
            while let (Some(a), Some(b)) = (left.first(), right.first()) {
                if a != b {
                    break;
                }
                left = &left[1..];
                right = &right[1..];
            }
            if left.first() < right.first() {
                std::mem::swap(&mut left, &mut right);
            }
            let (&value, tail) = left.split_first()?;
            left = tail;
            Some(value)
        })
    }

    fn remove_dominated(
        options: &mut Vec<Self>,
        player: Player,
        cache: &mut HashMap<GameValueNode, HashMap<GameValueNode, bool>>,
    ) {
        let mut retained = vec![true; options.len()];
        for i in 0..options.len() {
            let dominated = (0..options.len()).any(|j| {
                i != j
                    && retained[j]
                    && match player {
                        Player::Left => options[i].leq_memoized(&options[j], cache),
                        Player::Right => options[j].leq_memoized(&options[i], cache),
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
        Self::number_up_nimber(value, 0, 0)
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
        self.sum(&rhs, &mut HashMap::new(), &mut HashMap::new())
    }
}
impl Sub for GameValue {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        if self.0 == rhs.0 {
            Self::from(0)
        } else {
            self + -rhs
        }
    }
}
impl Sum for GameValue {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        let (mut number, mut up, mut nimber) = (Dyadic::from(0), 0, 0);
        let mut temperatures = Vec::new();
        let mut fingerprint = 0;
        let mut general = Self::from(0);
        for value in iter {
            if let Some((q, k, n, ts)) = value.as_switch_sum() {
                number = number + q;
                up += k;
                nimber ^= n;
                temperatures.extend_from_slice(ts);
                fingerprint ^= value.switch_hash();
            } else {
                general = general + value;
            }
        }
        temperatures.sort_unstable_by(|a, b| b.cmp(a));
        let mut retained = 0;
        for i in 0..temperatures.len() {
            if retained > 0 && temperatures[retained - 1] == temperatures[i] {
                retained -= 1;
            } else {
                temperatures[retained] = temperatures[i];
                retained += 1;
            }
        }
        temperatures.truncate(retained);
        let value = Self::from_switches(number, up, nimber, temperatures, fingerprint);
        if general.as_nimber() == Some(0) {
            value
        } else {
            value + general
        }
    }
}
impl PartialEq for GameValue {
    fn eq(&self, rhs: &Self) -> bool {
        match (&self.0, &rhs.0) {
            (GameValueNode::NumberUpNimber(a, k, n), GameValueNode::NumberUpNimber(b, l, m)) => {
                return a == b && k == l && n == m;
            }
            (GameValueNode::Switches(_), GameValueNode::Switches(_)) => {
                return self.0 == rhs.0;
            }
            _ => {}
        }
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
        if let Some((q, k, n, temperatures)) = self.as_switch_sum() {
            let mut separator = "";
            if q.numerator() != 0 || k == 0 && n == 0 && temperatures.is_empty() {
                write!(f, "{q}")?;
                separator = "+";
            }
            if k != 0 {
                f.write_str(separator)?;
                if k.unsigned_abs() != 1 {
                    write!(f, "{}", k.unsigned_abs())?;
                }
                f.write_str(if k > 0 { "↑" } else { "↓" })?;
                separator = "+";
            }
            if n != 0 {
                write!(f, "{separator}*")?;
                if n != 1 {
                    write!(f, "{n}")?;
                }
                separator = "+";
            }
            for t in temperatures {
                write!(f, "{separator}{{{t}|{}}}", -*t)?;
                separator = "+";
            }
            return Ok(());
        }
        f.write_str("{")?;
        for (separator, player) in [("", Player::Left), ("|", Player::Right)] {
            f.write_str(separator)?;
            for (i, g) in self.options(player).enumerate() {
                if i > 0 {
                    f.write_str(",")?;
                }
                write!(f, "{g}")?;
            }
        }
        f.write_str("}")
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
    use crate::tools::{
        Xorshift,
        testutil::{integer_boundary_values, structured_sequences},
    };

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
                for k in [-8192, -2, -1, 0, 1, 2, 8192, rng.random(-8192..=8192)] {
                    let value = GameValue::number_up_nimber(q, k, n);
                    let other = GameValue::number_up_nimber(r, -k, m);
                    assert_eq!((-value.clone()).as_number_up_nimber(), Some((-q, -k, n)));
                    assert_eq!(
                        (value.clone() + other).as_number_nimber(),
                        Some((q + r, n ^ m))
                    );
                    if k != 0 {
                        assert_eq!(
                            GameValue::from_options(
                                value.options(Player::Left),
                                value.options(Player::Right)
                            )
                            .as_number_up_nimber(),
                            Some((q, k, n)),
                        );
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
        let up = games.len();
        games.push((vec![0], vec![3]));
        let down = games.len();
        games.push((vec![3], vec![0]));
        let inner = games.len();
        games.push((vec![0], vec![2]));
        let tiny = games.len();
        games.push((vec![0], vec![inner]));
        let inner = games.len();
        games.push((vec![1], vec![0]));
        let miny = games.len();
        games.push((vec![inner], vec![0]));
        let products_start = games.len();
        let mut components = vec![0, 1, 2, 3, up, down, start + 2];
        let mut switches = vec![0];
        // Half-integer switches and their boundary numbers, with both signs of up.
        for t in 1..=3 {
            let a = start + indices[&t] * 8;
            let b = start + indices[&-t] * 8;
            let switch = games.len();
            games.push((vec![a], vec![b]));
            components.extend([a, b, switch]);
            switches.push(switch);
        }
        let mut states: Vec<Vec<_>> = structured_sequences(&mut rng, 0..components.len(), 0..=5)
            .map(|s| s.into_iter().map(|i| components[i]).collect())
            .collect();
        // At either numeric boundary, the up/nimber tail and turn parity decide.
        for (a, &first) in switches.iter().enumerate() {
            for (b, &second) in switches[..=a].iter().enumerate() {
                for q in [a as i64 - b as i64, b as i64 - a as i64] {
                    for u in [0, up, down] {
                        for n in [0, 3] {
                            states.push(vec![start + indices[&q] * 8, u, n, first, second]);
                        }
                    }
                }
            }
        }
        for &component in &components {
            let parts = [tiny, component, miny, component, tiny];
            states.extend((3..=5).map(|len| parts[..len].to_vec()));
        }
        // Translate each sampled DAG and Tiny/Miny by every existing half-integer.
        states.extend(
            (260..start)
                .chain([tiny, miny])
                .flat_map(|g| (0..numbers.len()).map(move |i| vec![g, start + i * 8])),
        );
        let switch_products_start = states.len();
        states.extend(
            (0usize..1 << (switches.len() - 1))
                .filter(|mask| mask.count_ones() > 1)
                .map(|mask| {
                    switches[1..]
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &s)| (mask >> i & 1 != 0).then_some(s))
                        .collect()
                }),
        );
        let mut products = Vec::new();
        let mut product_cache = HashMap::new();
        for parts in states {
            let root = parts
                .iter()
                .fold(0, |a, &b| sum_options(&mut games, a, b, &mut product_cache));
            products.push((root, parts));
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
        assert!(values[tiny].as_switch_sum().is_none());
        for (root, parts) in &products {
            let sum: GameValue = parts.iter().map(|&i| values[i].clone()).sum();
            assert_eq!(sum, values[*root]);
            if parts.iter().all(|&i| values[i].as_switch_sum().is_some()) {
                assert!(sum.as_switch_sum().is_some());
                assert!(values[*root].as_switch_sum().is_some(), "{}", values[*root]);
            }
            let reversed: GameValue = parts.iter().rev().map(|&i| values[i].clone()).sum();
            assert_eq!(sum, reversed);
            let folded = parts
                .iter()
                .fold(GameValue::from(0), |s, &i| s + values[i].clone());
            assert_eq!(sum, folded);
            // Check arithmetic results with the same raw oracle and canonical-form checks.
            values[*root] = sum;
        }
        let zero = GameValue::from(0);
        let mut differences = HashMap::new();
        let mut sums = HashMap::new();
        let roots = (0..products_start).chain(products.iter().map(|&(i, _)| i));
        for i in roots {
            let g = &values[i];
            let (left, right) = &games[i];
            let candidate = GameValue(GameValueNode::Options(
                0.into(),
                Rc::new((
                    left.iter().map(|&j| values[j].clone()).collect(),
                    right.iter().map(|&j| values[j].clone()).collect(),
                )),
            ));
            let winning = [false, true].map(|right| wins(&games, i, 0, right, false, &mut sums));
            for (player, win) in [Player::Left, Player::Right].into_iter().zip(winning) {
                assert_eq!(g.wins(player), win);
                let options: Vec<_> = g.options(player).collect();
                for (j, option) in options.iter().enumerate() {
                    if let Some((q, k, n, ts)) = option.as_switch_sum() {
                        let rebuilt: GameValue =
                            std::iter::once(GameValue::number_up_nimber(q, k, n))
                                .chain(
                                    ts.iter().map(|&t| {
                                        GameValue::from_options([t.into()], [(-t).into()])
                                    }),
                                )
                                .sum();
                        let memo = HashMap::from([(option.0.clone(), ())]);
                        assert!(memo.contains_key(&rebuilt.0));
                    }
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
            let mut partners: Vec<_> = if i < 260 {
                (0..260).collect()
            } else if i >= products_start {
                components
                    .iter()
                    .copied()
                    .chain([tiny])
                    .chain(products[switch_products_start..].iter().map(|&(i, _)| i))
                    .collect()
            } else if i >= asymmetric {
                (start..symmetric).collect()
            } else if i >= start {
                (start..asymmetric).collect()
            } else {
                (0..32).map(|_| rng.random(0..products_start)).collect()
            };
            if (260..start).contains(&i) {
                partners.push(i);
            }
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
                assert_eq!(
                    candidate.partial_cmp(&values[j]),
                    expected,
                    "raw options {i} vs {j}"
                );
                assert_eq!(g == &values[j], expected == Some(Ordering::Equal));
                if (i < 260 && j >= 4) || (asymmetric..products_start).contains(&i) {
                    continue;
                }
                assert_eq!((g.clone() - values[j].clone()).partial_cmp(&zero), expected);
                let sum = g.clone() + values[j].clone();
                assert_eq!(sum, values[j].clone() + g.clone());
                let collected: GameValue = [g.clone(), values[j].clone()].into_iter().sum();
                assert_eq!(sum, collected);
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

    // Build raw disjunctive sums without simplifying options or using game values.
    fn sum_options(
        games: &mut Vec<(Vec<usize>, Vec<usize>)>,
        a: usize,
        b: usize,
        cache: &mut HashMap<(usize, usize), usize>,
    ) -> usize {
        if a == 0 {
            return b;
        }
        if b == 0 {
            return a;
        }
        if let Some(&i) = cache.get(&(a, b)) {
            return i;
        }
        let (al, ar) = games[a].clone();
        let (bl, br) = games[b].clone();
        let mut options = [Vec::new(), Vec::new()];
        for (moves, (left, right)) in options.iter_mut().zip([(al, bl), (ar, br)]) {
            moves.extend(left.into_iter().map(|c| sum_options(games, c, b, cache)));
            moves.extend(right.into_iter().map(|c| sum_options(games, a, c, cache)));
        }
        let i = games.len();
        let [left, right] = options;
        games.push((left, right));
        cache.insert((a, b), i);
        i
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
