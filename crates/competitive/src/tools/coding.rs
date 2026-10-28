use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, VecDeque},
    hash::Hash,
    iter::repeat_with,
    mem::size_of,
};
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

#[cfg_attr(verus_keep_ghost, verus_verify)]
#[cfg_attr(verus_keep_ghost, verus_spec(result =>
    ensures result@ == unescaped(bytes@), result@.len() <= bytes@.len(),
))]
pub fn unescape(bytes: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(bytes.len());
    let mut escape = false;
    #[cfg(verus_keep_ghost)]
    proof! { assert(bytes@.skip(0) =~= bytes@); }
    #[cfg_attr(verus_keep_ghost, verus_spec(iter =>
        invariant buf@.len() <= iter.index(),
            unescaped(bytes@) == buf@ + decode_rest(bytes@.skip(iter.index()), escape),
    ))]
    for b in bytes {
        #[cfg(verus_keep_ghost)]
        proof_decl! { let ghost pending = escape; }
        match (escape, *b) {
            (true, b'n') => {
                buf.push(b'\n');
                escape = false;
            }
            (true, b'r') => {
                buf.push(b'\r');
                escape = false;
            }
            (true, b't') => {
                buf.push(b'\t');
                escape = false;
            }
            (true, b'\\') => {
                buf.push(b'\\');
                escape = false;
            }
            (true, b'0') => {
                buf.push(b'\0');
                escape = false;
            }
            (true, b'"') => {
                buf.push(b'"');
                escape = false;
            }
            (false, b'\\') => {
                escape = true;
            }
            (_, b) => {
                buf.push(b);
                escape = false;
            }
        }
        #[cfg(verus_keep_ghost)]
        proof! {
            let rest = bytes@.skip(iter.index());
            let tail = bytes@.skip(iter.index() + 1);
            assert(rest.skip(1) =~= tail);
            decode_head(rest, pending);
            assert(unescaped(bytes@) =~= buf@ + decode_rest(tail, escape));
        }
    }
    #[cfg(verus_keep_ghost)]
    proof! {
        #[verifier::proof_note("For arbitrary byte strings, unescape decodes backslash n/r/t/backslash/0/quote, removes the backslash before any other byte, drops a final unmatched backslash, and never lengthens the input")]
        assert(buf@ == unescaped(bytes@) && buf@.len() <= bytes@.len());
    }
    buf
}

fn to_bytestring(bytes: &[u8]) -> String {
    let mut count = [0usize; 128];
    let mut c = 0u8;
    let mut w = 0u32;
    for &b in bytes {
        w += 1;
        count[(c | (b >> w)) as usize] += 1;
        if w == 7 {
            count[(b & 0b1111111u8) as usize] += 1;
            c = 0;
            w = 0;
        } else {
            c = (b << (7 - w)) & 0b1111111u8;
        }
    }
    if w > 0 {
        count[c as usize] += 1;
        c = 0;
        w = 0;
    }
    let mut salt = 0u8;
    let mut extra_min = !0usize;
    for s in 0u8..128 {
        let mut extra = 0usize;
        for &b in b"\n\r\t\\\0\"" {
            extra += count[(b ^ s) as usize];
            extra += (b == s) as usize;
        }
        if extra < extra_min {
            extra_min = extra;
            salt = s;
        }
    }
    let cap = extra_min + count.iter().sum::<usize>() + 1;
    let mut buf = String::with_capacity(cap);
    macro_rules! escape_branch {
        ($e:literal) => {{
            buf.push('\\');
            buf.push($e);
        }};
    }
    macro_rules! escape {
        ($b:expr) => {
            match ($b) ^ salt {
                b'\n' => escape_branch!('n'),
                b'\r' => escape_branch!('r'),
                b'\t' => escape_branch!('t'),
                b'\\' => escape_branch!('\\'),
                b'\0' => escape_branch!('0'),
                // b'\'' => escape_branch!('\''),
                b'"' => escape_branch!('"'),
                b => buf.push(b as char),
            }
        };
    }
    escape!(0);
    for &b in bytes {
        w += 1;
        escape!(c | (b >> w));
        if w == 7 {
            escape!(b & 0b1111111u8);
            c = 0;
            w = 0;
        } else {
            c = (b << (7 - w)) & 0b1111111u8;
        }
    }
    if w > 0 {
        escape!(c);
    }
    assert_eq!(cap, buf.len());
    buf
}

fn from_bytestring(bytes: &[u8]) -> Vec<u8> {
    assert!(!bytes.is_empty());
    let cap = (bytes.len() - 1) * 7 / 8;
    let mut buf = Vec::with_capacity(cap);
    let salt = bytes[0];
    let bytes = &bytes[1..];
    let mut c = 0u8;
    let mut w = 0u32;
    for &b in bytes {
        let b = b ^ salt;
        if w == 0 {
            c = b << 1;
            w = 7;
        } else {
            w -= 1;
            buf.push(c | (b >> w));
            c = if w > 0 { b << (8 - w) } else { 0 };
        }
    }
    assert_eq!(cap, buf.len());
    buf
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum HuffmanTree {
    Leaf(u8),
    Node(Box<HuffmanTree>, Box<HuffmanTree>),
}

#[cfg_attr(verus_keep_ghost, verus_verify)]
#[derive(Debug)]
struct BitWriter {
    bytes: Vec<u8>,
    last: u8,
    w: u32,
}

#[cfg_attr(verus_keep_ghost, verus_verify)]
impl Default for BitWriter {
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        ensures result.well_formed(), result@ == Seq::<bool>::empty(),
    ))]
    fn default() -> Self {
        #[cfg(verus_keep_ghost)]
        proof! { zero_byte(); }
        Self {
            bytes: Default::default(),
            last: Default::default(),
            w: 8,
        }
    }
}

impl BitWriter {
    #[cfg_attr(verus_keep_ghost, verus_verify)]
    #[cfg_attr(verus_keep_ghost, verus_spec(
        requires old(self).well_formed(), old(self).bytes@.len() < usize::MAX,
        ensures final(self).well_formed(), final(self)@ == old(self)@.push(b),
    ))]
    fn push_bit(&mut self, b: bool) {
        #[cfg(verus_keep_ghost)]
        proof_decl! { let ghost before = *self; }
        self.w -= 1;
        self.last |= (b as u8) << self.w;
        #[cfg(verus_keep_ghost)]
        proof! {
            assert forall|i: int| 0 <= i < 8 implies
                #[trigger] byte_bit(self.last, i) == if i == 8 - before.w { b } else { byte_bit(before.last, i) } by {
                byte_set(before.last, b, (8 - before.w) as u8, i as u8);
            }
        }
        if self.w == 0 {
            self.bytes.push(self.last);
            self.w = 8;
            self.last = 0;
        }
        #[cfg(verus_keep_ghost)]
        proof! {
            zero_byte();
            assert(self@ =~= before@.push(b));
        }
    }
    #[cfg_attr(verus_keep_ghost, verus_verify)]
    #[cfg_attr(verus_keep_ghost, verus_spec(
        requires old(self).well_formed(), old(self).bytes@.len() < usize::MAX,
        ensures final(self).well_formed(), final(self)@ == old(self)@ + byte_bits(seq![b]),
    ))]
    fn push_u8(&mut self, b: u8) {
        #[cfg(verus_keep_ghost)]
        proof_decl! { let ghost before = *self; }
        self.bytes.push(self.last | (b >> (8 - self.w)));
        if self.w < 8 {
            self.last = b << self.w;
        }
        #[cfg(verus_keep_ghost)]
        proof! {
            assert forall|i: int| 0 <= i < 8 implies
                #[trigger] byte_bit(self.bytes@[before.bytes@.len() as int], i)
                    == if i < 8 - before.w { byte_bit(before.last, i) } else { byte_bit(b, i - (8 - before.w)) } by {
                byte_append(before.last, b, before.w as u8, i as u8);
            }
            assert forall|i: int| 0 <= i < 8 implies #[trigger] byte_bit(self.last, i)
                    == if i < 8 - before.w { byte_bit(b, i + before.w) } else { false } by {
                byte_append(before.last, b, before.w as u8, i as u8);
            }
            assert(self@ =~= before@ + byte_bits(seq![b]));
        }
    }
    #[cfg_attr(verus_keep_ghost, verus_verify)]
    #[cfg_attr(verus_keep_ghost, verus_spec(
        requires old(self).well_formed(), old(self).bytes@.len() + 8 <= usize::MAX,
            c <= 64, c == 64 || b >> c == 0,
        ensures final(self).well_formed(), final(self)@ == old(self)@ + word_bits(b, c as nat),
    ))]
    fn push_u64(&mut self, b: u64, mut c: u32) {
        #[cfg(verus_keep_ghost)]
        proof_decl! {
            let ghost before = *self;
            let ghost count = c;
            let ghost expected = before@ + word_bits(b, count as nat);
        }
        let k = self.w.min(c);
        self.w -= k;
        c -= k;
        self.last |= (b >> c << self.w) as u8;
        #[cfg(verus_keep_ghost)]
        proof! {
            assert forall|i: int| 0 <= i < 8 implies #[trigger] byte_bit(self.last, i)
                == if i < 8 - before.w { byte_bit(before.last, i) }
                    else if i < 8 - self.w { word_bit(b, count - 1 - (i - (8 - before.w))) }
                    else { false } by {
                word_prefix(before.last, b, before.w, count, i as u32);
            }
        }
        if self.w == 0 {
            self.bytes.push(self.last);
            let (s, t) = (c / 8, c % 8);
            #[cfg(verus_keep_ghost)]
            proof! {
                assert(byte_bits(self.bytes@) =~= expected.take(self.bytes@.len() as int * 8));
            }
            #[cfg_attr(verus_keep_ghost, verus_spec(iter =>
                invariant before.well_formed(), count <= 64, before.w <= count,
                    count == 64 || b >> count == 0,
                    expected == before@ + word_bits(b, count as nat),
                    s == (count - before.w) / 8, t == (count - before.w) % 8,
                    c == count - before.w - 8 * iter.index(),
                    self.w == 0,
                    self.bytes@.len() == before.bytes@.len() + 1 + iter.index(),
                    byte_bits(self.bytes@) == expected.take(self.bytes@.len() as int * 8),
            ))]
            for _ in 0..s {
                c -= 8;
                self.bytes.push((b >> c) as u8);
                #[cfg(verus_keep_ghost)]
                proof! {
                    assert forall|i: int| 0 <= i < 8 implies #[trigger] byte_bit((b >> c) as u8, i)
                        == word_bit(b, c + 7 - i) by {
                        word_byte(b, c, i as u32);
                    }
                    assert(byte_bits(self.bytes@) =~= expected.take(self.bytes@.len() as int * 8));
                }
            }
            self.last = 0;
            self.w = 8 - t;
            if t > 0 {
                self.last = (b as u8) << self.w;
            }
            #[cfg(verus_keep_ghost)]
            proof! {
                zero_byte();
                assert(b >> 0u32 == b) by(bit_vector);
                assert forall|i: int| 0 <= i < 8 implies #[trigger] byte_bit(self.last, i)
                    == if i < t { word_bit(b, t - 1 - i) } else { false } by {
                    byte_append(0, b as u8, self.w as u8, i as u8);
                    if i < t { word_byte(b, 0, (i + self.w) as u32); }
                }
            }
        }
        #[cfg(verus_keep_ghost)]
        proof! {
            assert(self@.len() == expected.len());
            assert forall|i: int| 0 <= i < self@.len() implies #[trigger] self@[i] == expected[i] by {
                if i < self.bytes@.len() * 8 {
                    assert(byte_bits(self.bytes@)[i] == expected[i]);
                }
            }
            assert(self@ =~= expected);
        }
    }
    #[cfg_attr(verus_keep_ghost, verus_verify)]
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires self.well_formed(), self.bytes@.len() < usize::MAX,
        ensures result@.len() == (self@.len() + 7) / 8,
            byte_bits(result@).take(self@.len() as int) == self@,
            forall|i: int| self@.len() <= i < result@.len() * 8
                ==> !#[trigger] byte_bits(result@)[i],
    ))]
    fn into_inner(mut self) -> Vec<u8> {
        #[cfg(verus_keep_ghost)]
        proof_decl! { let ghost before = self; }
        if self.w < 8 {
            self.bytes.push(self.last);
        }
        #[cfg(verus_keep_ghost)]
        proof! {
            assert(byte_bits(self.bytes@).take(before@.len() as int) =~= before@);
        }
        self.bytes
    }
}

#[cfg_attr(verus_keep_ghost, verus_verify)]
#[derive(Debug)]
struct BitReader<'a> {
    bytes: &'a [u8],
    pos: u32,
}

#[cfg_attr(verus_keep_ghost, verus_verify)]
impl<'a> BitReader<'a> {
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        ensures result.well_formed(), result@ == byte_bits(bytes@),
    ))]
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires old(self).well_formed(), old(self)@.len() > 0,
        ensures final(self).well_formed(),
            result == old(self)@[0], final(self)@ == old(self)@.drop_first(),
    ))]
    fn read_bit(&mut self) -> bool {
        let b = (self.bytes[0] >> (7 - self.pos)) & 1 == 1;
        self.pos += 1;
        if self.pos == 8 {
            self.pos = 0;
            self.bytes = &self.bytes[1..];
        }
        b
    }
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires old(self).well_formed(), old(self)@.len() >= 8,
        ensures final(self).well_formed(), final(self)@ == old(self)@.skip(8),
            forall|i: int| 0 <= i < 8 ==> #[trigger] byte_bit(result, i) == old(self)@[i],
    ))]
    fn read_u8(&mut self) -> u8 {
        #[cfg(verus_keep_ghost)]
        proof_decl! { let ghost before = *self; }
        let b = self.bytes[0] << self.pos;
        self.bytes = &self.bytes[1..];
        #[cfg(verus_keep_ghost)]
        proof! {
            let result = if self.pos == 0 { b } else { b | (self.bytes@[0] >> (8 - self.pos)) };
            let first = before.bytes@[0];
            let second = if before.pos == 0 { 0u8 } else { before.bytes@[1] };
            assert(first << 0u32 == first) by(bit_vector);
            assert(result == if before.pos == 0 { first } else {
                (first << before.pos) | (second >> ((8 - before.pos) as u32))
            });
            assert forall|i: int| 0 <= i < 8 implies
                #[trigger] byte_bit(result, i) == before@[i] by {
                byte_join(first, second, before.pos, i as u32);
            }
        }
        if self.pos == 0 {
            b
        } else {
            b | (self.bytes[0] >> (8 - self.pos))
        }
    }
}

#[cfg(verus_keep_ghost)]
verus! {
    spec fn escaped_byte(byte: u8) -> u8 {
        match byte {
            b'n' => b'\n', b'r' => b'\r', b't' => b'\t',
            b'0' => b'\0', other => other,
        }
    }

    pub closed spec fn unescaped(bytes: Seq<u8>) -> Seq<u8>
        decreases bytes.len(),
    {
        if bytes.len() == 0 { seq![] }
        else if bytes[0] == b'\\' {
            if bytes.len() == 1 { seq![] }
            else { seq![escaped_byte(bytes[1])] + unescaped(bytes.skip(2)) }
        } else { seq![bytes[0]] + unescaped(bytes.skip(1)) }
    }

    spec fn decode_rest(bytes: Seq<u8>, pending: bool) -> Seq<u8> {
        if pending {
            if bytes.len() == 0 { seq![] }
            else { seq![escaped_byte(bytes[0])] + unescaped(bytes.skip(1)) }
        } else { unescaped(bytes) }
    }

    proof fn decode_head(bytes: Seq<u8>, pending: bool)
        requires bytes.len() > 0,
        ensures decode_rest(bytes, pending) == if !pending && bytes[0] == b'\\' {
            decode_rest(bytes.skip(1), true)
        } else {
            seq![if pending { escaped_byte(bytes[0]) } else { bytes[0] }] + decode_rest(bytes.skip(1), false)
        },
    {
        if !pending && bytes[0] == b'\\' && bytes.len() > 1 {
            assert(bytes.skip(1).skip(1) =~= bytes.skip(2));
        }
    }

    spec fn byte_bit(byte: u8, index: int) -> bool {
        (byte >> ((7 - index) as u8)) & 1u8 == 1u8
    }

    spec fn byte_bits(bytes: Seq<u8>) -> Seq<bool> {
        Seq::new(bytes.len() * 8, |i: int| byte_bit(bytes[i / 8], i % 8))
    }

    spec fn word_bit(word: u64, index: int) -> bool {
        (word >> (index as u32)) & 1u64 == 1u64
    }

    spec fn word_bits(word: u64, count: nat) -> Seq<bool> {
        Seq::new(count, |i: int| word_bit(word, count - 1 - i))
    }

    proof fn word_prefix(prefix: u8, word: u64, free: u32, count: u32, index: u32)
        requires 1 <= free <= 8, count <= 64, index < 8,
            count == 64 || word >> count == 0,
            index >= 8 - free ==> !byte_bit(prefix, index as int),
        ensures ({
            let taken = if free < count { free } else { count };
            let remaining = (count - taken) as u32;
            let space = (free - taken) as u32;
            byte_bit(prefix | ((word >> remaining << space) as u8), index as int)
                == if index < 8 - free { byte_bit(prefix, index as int) }
                    else if index < 8 - space { word_bit(word, count - 1 - (index - (8 - free))) }
                    else { false }
        }),
    {
        let taken = if free < count { free } else { count };
        let remaining = (count - taken) as u32;
        let space = (free - taken) as u32;
        let shift = (count - 1 - (index - (8 - free))) as u32;
        assert((word >> shift) & 1u64 <= 1u64) by(bit_vector);
        assert((prefix >> ((7u32 - index) as u32)) & 1u8 <= 1u8) by(bit_vector);
        assert(((prefix | ((word >> remaining << space) as u8)) >> ((7u32 - index) as u32)) & 1u8
            == if index < 8u32 - free { (prefix >> ((7u32 - index) as u32)) & 1u8 }
                else if index < 8u32 - space {
                    ((word >> ((count - 1u32 - (index - (8u32 - free))) as u32)) & 1u64) as u8
                } else { 0u8 }) by(bit_vector)
            requires 1 <= free <= 8, count <= 64, index < 8,
                count == 64 || word >> count == 0,
                index >= 8u32 - free ==> (prefix >> ((7u32 - index) as u32)) & 1u8 == 0,
                taken == if free < count { free } else { count },
                remaining == (count - taken) as u32,
                space == (free - taken) as u32;
    }

    proof fn word_byte(word: u64, shift: u32, index: u32)
        requires shift < 64, index < 8,
        ensures byte_bit((word >> shift) as u8, index as int)
            == word_bit(word, shift + 7 - index),
    {
        let bit_index = (shift + 7 - index) as u32;
        assert((word >> bit_index) & 1u64 <= 1u64) by(bit_vector);
        assert((((word >> shift) as u8) >> ((7u32 - index) as u32)) & 1u8
            == ((word >> ((shift + 7u32 - index) as u32)) & 1u64) as u8) by(bit_vector)
            requires shift < 64, index < 8;
    }

    proof fn zero_byte()
        ensures forall|i: int| 0 <= i < 8 ==> !#[trigger] byte_bit(0u8, i),
    {
        assert forall|i: int| 0 <= i < 8 implies !#[trigger] byte_bit(0u8, i) by {
            let shift = (7 - i) as u8;
            assert((0u8 >> shift) & 1u8 == 0u8) by(bit_vector);
        }
    }

    proof fn equal_bytes(first: u8, second: u8)
        requires forall|i: int| 0 <= i < 8 ==> #[trigger] byte_bit(first, i) == byte_bit(second, i),
        ensures first == second,
    {
        assert(byte_bit(first, 0) == byte_bit(second, 0));
        assert(byte_bit(first, 1) == byte_bit(second, 1));
        assert(byte_bit(first, 2) == byte_bit(second, 2));
        assert(byte_bit(first, 3) == byte_bit(second, 3));
        assert(byte_bit(first, 4) == byte_bit(second, 4));
        assert(byte_bit(first, 5) == byte_bit(second, 5));
        assert(byte_bit(first, 6) == byte_bit(second, 6));
        assert(byte_bit(first, 7) == byte_bit(second, 7));
        assert(first == second) by(bit_vector)
            requires
                ((first >> 7u8) & 1u8 == 1) == ((second >> 7u8) & 1u8 == 1),
                ((first >> 6u8) & 1u8 == 1) == ((second >> 6u8) & 1u8 == 1),
                ((first >> 5u8) & 1u8 == 1) == ((second >> 5u8) & 1u8 == 1),
                ((first >> 4u8) & 1u8 == 1) == ((second >> 4u8) & 1u8 == 1),
                ((first >> 3u8) & 1u8 == 1) == ((second >> 3u8) & 1u8 == 1),
                ((first >> 2u8) & 1u8 == 1) == ((second >> 2u8) & 1u8 == 1),
                ((first >> 1u8) & 1u8 == 1) == ((second >> 1u8) & 1u8 == 1),
                ((first >> 0u8) & 1u8 == 1) == ((second >> 0u8) & 1u8 == 1);
    }

    proof fn byte_set(byte: u8, bit: bool, position: u8, index: u8)
        requires position < 8, index < 8, !byte_bit(byte, position as int),
        ensures byte_bit(byte | ((bit as u8) << ((7 - position) as u8)), index as int)
            == if index == position { bit } else { byte_bit(byte, index as int) },
    {
        assert((byte >> ((7u8 - position) as u8)) & 1u8 <= 1u8) by(bit_vector);
        assert(((byte | ((bit as u8) << ((7u8 - position) as u8))) >> ((7u8 - index) as u8)) & 1u8
            == if index == position { bit as u8 } else { (byte >> ((7u8 - index) as u8)) & 1u8 }) by(bit_vector)
            requires position < 8, index < 8, (byte >> ((7u8 - position) as u8)) & 1u8 == 0;
    }

    proof fn byte_append(prefix: u8, byte: u8, free: u8, index: u8)
        requires 1 <= free <= 8, index < 8,
            index >= 8 - free ==> !byte_bit(prefix, index as int),
        ensures byte_bit(prefix | (byte >> ((8 - free) as u8)), index as int)
                == if index < 8 - free { byte_bit(prefix, index as int) }
                    else { byte_bit(byte, index - (8 - free)) },
            byte_bit(byte << free, index as int)
                == if index < 8 - free { byte_bit(byte, index + free) } else { false },
    {
        assert((prefix >> ((7u8 - index) as u8)) & 1u8 <= 1u8) by(bit_vector);
        assert(((prefix | (byte >> ((8u8 - free) as u8))) >> ((7u8 - index) as u8)) & 1u8
            == if index < 8u8 - free { (prefix >> ((7u8 - index) as u8)) & 1u8 }
                else { (byte >> ((15u8 - index - free) as u8)) & 1u8 }) by(bit_vector)
            requires 1 <= free <= 8, index < 8,
                index >= 8u8 - free ==> (prefix >> ((7u8 - index) as u8)) & 1u8 == 0;
        assert(((byte << free) >> ((7u8 - index) as u8)) & 1u8
            == if index < 8u8 - free { (byte >> ((7u8 - index - free) as u8)) & 1u8 } else { 0u8 }) by(bit_vector)
            requires 1 <= free <= 8, index < 8;
    }

    impl BitWriter {
        pub closed spec fn well_formed(&self) -> bool {
            1 <= self.w <= 8 && forall|i: int| 8 - self.w <= i < 8 ==> !#[trigger] byte_bit(self.last, i)
        }
    }

    impl View for BitWriter {
        type V = Seq<bool>;

        closed spec fn view(&self) -> Self::V {
            Seq::new((self.bytes@.len() * 8 + 8 - self.w) as nat, |i: int|
                if i < self.bytes@.len() * 8 { byte_bit(self.bytes@[i / 8], i % 8) }
                else { byte_bit(self.last, i - self.bytes@.len() * 8) })
        }
    }

    proof fn byte_join(first: u8, second: u8, offset: u32, index: u32)
        requires offset < 8, index < 8,
        ensures byte_bit(if offset == 0 { first } else {
                (first << offset) | (second >> ((8 - offset) as u32))
            }, index as int)
            == if index + offset < 8 {
                byte_bit(first, index + offset)
            } else {
                byte_bit(second, index + offset - 8)
            },
    {
        assert((if offset == 0 { first } else {
                (first << offset) | (second >> ((8u32 - offset) as u32))
            } >> ((7u32 - index) as u32)) & 1u8
            == if index + offset < 8 {
                (first >> ((7u32 - (index + offset)) as u32)) & 1u8
            } else {
                (second >> ((15u32 - (index + offset)) as u32)) & 1u8
            }) by(bit_vector)
            requires offset < 8, index < 8;
    }

    impl BitReader<'_> {
        spec fn well_formed(&self) -> bool {
            self.pos < 8 && (self.bytes@.len() == 0 ==> self.pos == 0)
        }
    }

    impl View for BitReader<'_> {
        type V = Seq<bool>;

        closed spec fn view(&self) -> Self::V {
            Seq::new((self.bytes@.len() * 8 - self.pos) as nat,
                |i: int| byte_bit(self.bytes@[(i + self.pos) / 8], (i + self.pos) % 8))
        }
    }
}

fn huffman_coding(bytes: &[u8]) -> Vec<u8> {
    fn make_table(t: &HuffmanTree, code: u64, len: u32, table: &mut [(u64, u32)]) {
        match t {
            HuffmanTree::Leaf(i) => {
                table[*i as usize] = (code, len.max(1));
            }
            HuffmanTree::Node(l, r) => {
                make_table(l, code << 1, len + 1, table);
                make_table(r, (code << 1) | 1, len + 1, table);
            }
        }
    }
    fn output_tree(t: &HuffmanTree, buf: &mut BitWriter) {
        match t {
            HuffmanTree::Leaf(i) => {
                buf.push_bit(false);
                buf.push_u8(*i);
            }
            HuffmanTree::Node(l, r) => {
                buf.push_bit(true);
                output_tree(l, buf);
                output_tree(r, buf);
            }
        }
    }

    let mut freq = [0usize; 256];
    for &b in &bytes.len().to_le_bytes() {
        freq[b as usize] += 1;
    }
    for &b in bytes {
        freq[b as usize] += 1;
    }
    let mut heap = BinaryHeap::new();
    for (i, &f) in freq.iter().enumerate() {
        if f > 0 {
            heap.push(Reverse((f, 0usize, HuffmanTree::Leaf(i as _))));
        }
    }
    let t = if heap.is_empty() {
        HuffmanTree::Node(
            Box::new(HuffmanTree::Leaf(0)),
            Box::new(HuffmanTree::Leaf(0)),
        )
    } else {
        loop {
            let Reverse((f, c, t)) = heap.pop().unwrap();
            if let Some(Reverse((ff, cc, tt))) = heap.pop() {
                heap.push(Reverse((
                    f + ff,
                    c.max(cc) + 1,
                    HuffmanTree::Node(Box::new(t), Box::new(tt)),
                )));
            } else {
                break t;
            }
        }
    };

    let mut table = vec![(0u64, 0u32); 256];
    make_table(&t, 0, 0, &mut table);
    let mut buf = BitWriter::default();
    output_tree(&t, &mut buf);
    for &b in &bytes.len().to_le_bytes() {
        let (x, y) = table[b as usize];
        buf.push_u64(x, y);
    }
    for &b in bytes {
        let (x, y) = table[b as usize];
        buf.push_u64(x, y);
    }
    buf.into_inner()
}

fn huffman_decoding(bytes: &[u8]) -> Vec<u8> {
    fn read_tree(reader: &mut BitReader) -> HuffmanTree {
        if reader.read_bit() {
            HuffmanTree::Node(Box::new(read_tree(reader)), Box::new(read_tree(reader)))
        } else {
            HuffmanTree::Leaf(reader.read_u8())
        }
    }
    fn decode(mut t: &HuffmanTree, reader: &mut BitReader) -> u8 {
        loop {
            match t {
                HuffmanTree::Leaf(i) => break *i,
                HuffmanTree::Node(l, r) => t = if reader.read_bit() { r } else { l },
            }
        }
    }

    let mut reader = BitReader::new(bytes);
    let t = read_tree(&mut reader);
    const C: usize = size_of::<usize>();
    let mut size: [u8; C] = [0u8; C];
    for b in &mut size {
        *b = decode(&t, &mut reader);
    }
    let size = usize::from_le_bytes(size);
    let mut buf = vec![];
    for i in 0..size {
        buf.push(decode(&t, &mut reader));
        if i < 10 {}
    }
    buf
}

pub trait SerdeByteStr {
    fn serialize(&self, buf: &mut Vec<u8>);

    fn deserialize<I>(iter: &mut I) -> Self
    where
        I: Iterator<Item = u8>;

    fn serialize_bytestr(&self) -> String {
        let mut bytes = vec![];
        self.serialize(&mut bytes);
        let bytes = huffman_coding(&bytes);
        to_bytestring(&bytes)
    }

    fn deserialize_from_bytes(bytes: &[u8]) -> Self
    where
        Self: Sized,
    {
        let bytes = from_bytestring(bytes);
        let bytes = huffman_decoding(&bytes);
        Self::deserialize(&mut bytes.iter().cloned())
    }
}

impl SerdeByteStr for bool {
    fn serialize(&self, buf: &mut Vec<u8>) {
        (*self as u8).serialize(buf)
    }
    fn deserialize<I>(iter: &mut I) -> Self
    where
        I: Iterator<Item = u8>,
    {
        iter.next().unwrap() != 0
    }
}

macro_rules! impl_serdebytestr_num {
    ($($t:ty)*) => {
        $(
            impl SerdeByteStr for $t {
                fn serialize(&self, buf: &mut Vec<u8>) {
                    buf.extend(self.to_le_bytes().iter());
                }
                fn deserialize<I>(iter: &mut I) -> Self
                where
                    I: Iterator<Item = u8>,
                {
                    const C: usize = size_of::<$t>();
                    let mut bytes: [u8; C] = [0u8; C];
                    for (b, i) in bytes.iter_mut().zip(iter) {
                        *b = i;
                    }
                    <$t>::from_le_bytes(bytes)
                }
            }
        )*
    };
}

impl_serdebytestr_num!(u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64);

impl SerdeByteStr for char {
    fn serialize(&self, buf: &mut Vec<u8>) {
        (*self as u32).serialize(buf)
    }
    fn deserialize<I>(iter: &mut I) -> Self
    where
        I: Iterator<Item = u8>,
    {
        char::from_u32(u32::deserialize(iter)).expect("invalid Unicode scalar value")
    }
}

impl SerdeByteStr for () {
    fn serialize(&self, _buf: &mut Vec<u8>) {}
    fn deserialize<I>(_iter: &mut I) -> Self
    where
        I: Iterator<Item = u8>,
    {
    }
}

macro_rules! impl_serdebytestr_tuple {
    (@impl $($A:ident $a:ident)*) => {
        impl<$($A,)*> SerdeByteStr for ($($A,)*)
        where
            $($A: SerdeByteStr),*
        {
            fn serialize(&self, buf: &mut Vec<u8>) {
                let ($($a,)*) = self;
                $(SerdeByteStr::serialize($a, buf));*
            }
            fn deserialize<Iter>(iter: &mut Iter) -> Self
            where
                Iter: Iterator<Item = u8>,
            {
                ($(<$A as SerdeByteStr>::deserialize(iter),)*)
            }
        }
    };
    (@inc , $B:ident $b:ident $($C:ident $c:ident)*) => {
        impl_serdebytestr_tuple!(@inc $B $b, $($C $c)*);
    };
    (@inc $($A:ident $a:ident)*, $B:ident $b:ident $($C:ident $c:ident)*) => {
        impl_serdebytestr_tuple!(@impl $($A $a)*);
        impl_serdebytestr_tuple!(@inc $($A $a)* $B $b, $($C $c)*);
    };
    (@inc $($A:ident $a:ident)*,) => {
        impl_serdebytestr_tuple!(@impl $($A $a)*);
    };
    ($($t:tt)*) => {
        impl_serdebytestr_tuple!(@inc , $($t)*);
    };
}
impl_serdebytestr_tuple!(A a B b C c D d E e F f G g H h I i J j K k);

impl<T> SerdeByteStr for Option<T>
where
    T: SerdeByteStr,
{
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.is_some().serialize(buf);
        if let Some(x) = self {
            x.serialize(buf);
        }
    }
    fn deserialize<I>(iter: &mut I) -> Self
    where
        I: Iterator<Item = u8>,
    {
        if bool::deserialize(iter) {
            Some(T::deserialize(iter))
        } else {
            None
        }
    }
}

impl SerdeByteStr for String {
    fn serialize(&self, buf: &mut Vec<u8>) {
        let bytes = self.bytes();
        bytes.len().serialize(buf);
        for x in bytes {
            x.serialize(buf);
        }
    }
    fn deserialize<I>(iter: &mut I) -> Self
    where
        I: Iterator<Item = u8>,
    {
        let n = usize::deserialize(iter);
        let bytes = match iter.size_hint() {
            (lower, Some(upper)) if lower == upper && lower >= n => iter.by_ref().take(n).collect(),
            _ => repeat_with(|| u8::deserialize(iter)).take(n).collect(),
        };
        String::from_utf8(bytes).expect("invalid UTF-8")
    }
}

macro_rules! impl_serdebytestr_seq {
    ($([$($g:ident)*] $t:ty $(where [$($tt:tt)*])?),* $(,)?) => {
        $(
            impl<$($g),*> SerdeByteStr for $t
            where
                $($g: SerdeByteStr,)*
                $($($tt)*)?
            {
                fn serialize(&self, buf: &mut Vec<u8>) {
                    self.len().serialize(buf);
                    for x in self {
                        x.serialize(buf);
                    }
                }
                fn deserialize<I>(iter: &mut I) -> Self
                where
                    I: Iterator<Item = u8>,
                {
                    let n = usize::deserialize(iter);
                    repeat_with(|| SerdeByteStr::deserialize(iter)).take(n).collect()
                }
            }
        )*
    };
    (@kv $([$($g:ident)*] $t:ty $(where [$($tt:tt)*])?),* $(,)?) => {
        $(
            impl<$($g),*> SerdeByteStr for $t
            where
                $($g: SerdeByteStr,)*
                $($($tt)*)?
            {
                fn serialize(&self, buf: &mut Vec<u8>) {
                    self.len().serialize(buf);
                    for (k, v) in self {
                        k.serialize(buf);
                        v.serialize(buf);
                    }
                }
                fn deserialize<I>(iter: &mut I) -> Self
                where
                    I: Iterator<Item = u8>,
                {
                    let n = usize::deserialize(iter);
                    repeat_with(|| SerdeByteStr::deserialize(iter)).take(n).collect()
                }
            }
        )*
    };
}

impl_serdebytestr_seq!(
    [T] Vec<T>,
    [T] VecDeque<T>,
    [T] BinaryHeap<T> where [T: Ord],
    [T] BTreeSet<T> where [T: Ord],
    [T] HashSet<T> where [T: Eq + Hash],
);
impl_serdebytestr_seq!(
    @kv
    [K V] BTreeMap<K, V> where [K: Ord],
    [K V] HashMap<K, V> where [K: Eq + Hash],
);

#[cfg(verus_keep_ghost)]
mod verification {
    use super::*;

    verus! {
        fn bit_roundtrip(bits: &[bool]) {
            proof {
                reveal(BitWriter::well_formed);
                reveal(<BitWriter as View>::view);
            }
            let mut writer = BitWriter::default();
            let mut index = 0;
            while index < bits.len()
                invariant index <= bits.len(), writer.well_formed(),
                    writer@ == bits@.take(index as int),
                decreases bits.len() - index,
            {
                assert(writer.bytes@.len() * 8 <= writer@.len());
                writer.push_bit(bits[index]);
                index += 1;
                assert(writer@ =~= bits@.take(index as int));
            }
            assert(bits@.take(bits@.len() as int) =~= bits@);
            let encoded = writer.into_inner();
            let ghost packed = byte_bits(encoded@);
            let mut reader = BitReader::new(&encoded);
            let mut index = 0;
            while index < bits.len()
                invariant index <= bits.len(), reader.well_formed(),
                    reader@ == packed.skip(index as int),
                    packed.len() >= bits.len(), packed.take(bits.len() as int) == bits@,
                decreases bits.len() - index,
            {
                let bit = reader.read_bit();
                #[verifier::proof_note("For every finite bit slice, writing its bits and reading them back returns each original bit in order, including byte boundaries and empty input")]
                assert(bit == bits@[index as int]);
                index += 1;
                assert(reader@ =~= packed.skip(index as int));
            }
            #[verifier::proof_note("Finishing a bit stream uses the minimum whole-byte length and pads only the final byte with fewer than eight zero bits")]
            assert(encoded@.len() == (bits@.len() + 7) / 8
                && reader@.len() < 8
                && forall|i: int| 0 <= i < reader@.len() ==> !#[trigger] reader@[i]);
        }

        fn mixed_roundtrip(prefix: &[bool], byte: u8, word: u64, count: u32)
            requires count <= 64, count == 64 || word >> count == 0,
        {
            proof {
                reveal(BitWriter::well_formed);
                reveal(<BitWriter as View>::view);
            }
            let mut writer = BitWriter::default();
            let mut index = 0;
            while index < prefix.len()
                invariant index <= prefix.len(), writer.well_formed(),
                    writer@ == prefix@.take(index as int),
                decreases prefix.len() - index,
            {
                assert(writer.bytes@.len() * 8 <= writer@.len());
                writer.push_bit(prefix[index]);
                index += 1;
                assert(writer@ =~= prefix@.take(index as int));
            }
            assert(writer@ == prefix@);
            writer.push_u8(byte);
            assert(writer@.len() == prefix@.len() + 8);
            assert(writer.bytes@.len() * 8 <= writer@.len());
            writer.push_u64(word, count);
            assert(writer@.len() == prefix@.len() + 8 + count);
            assert(writer.bytes@.len() * 8 <= writer@.len());
            let encoded = writer.into_inner();
            let ghost packed = byte_bits(encoded@);
            let mut reader = BitReader::new(&encoded);
            let mut index = 0;
            while index < prefix.len()
                invariant index <= prefix.len(), reader.well_formed(),
                    reader@ == packed.skip(index as int),
                    packed.len() >= prefix.len() + 8 + count,
                    packed.take(prefix.len() + 8 + count) == prefix@ + byte_bits(seq![byte]) + word_bits(word, count as nat),
                decreases prefix.len() - index,
            {
                let bit = reader.read_bit();
                assert(packed.take(prefix.len() + 8 + count)[index as int] == packed[index as int]);
                assert(bit == prefix@[index as int]);
                index += 1;
                assert(reader@ =~= packed.skip(index as int));
            }
            let decoded = reader.read_u8();
            proof {
                assert forall|i: int| 0 <= i < 8 implies #[trigger] byte_bit(decoded, i) == byte_bit(byte, i) by {
                    assert(packed.take(prefix.len() + 8 + count)[prefix.len() + i] == packed[prefix.len() + i]);
                }
                equal_bytes(decoded, byte);
                #[verifier::proof_note("After every arbitrary-length bit prefix, an appended byte is recovered exactly by the unaligned byte reader without changing the preceding bits")]
                assert(decoded == byte);
            }
            let mut index = 0;
            while index < count
                invariant index <= count, reader.well_formed(),
                    reader@ == packed.skip(prefix.len() + 8 + index),
                    packed.len() >= prefix.len() + 8 + count,
                    packed.take(prefix.len() + 8 + count) == prefix@ + byte_bits(seq![byte]) + word_bits(word, count as nat),
                decreases count - index,
            {
                let bit = reader.read_bit();
                assert(packed.take(prefix.len() + 8 + count)[prefix.len() + 8 + index]
                    == packed[prefix.len() + 8 + index]);
                #[verifier::proof_note("For every u64 value fitting a requested width from zero through 64 bits, push_u64 appends exactly that most-significant-first bit sequence after arbitrary preceding bits and a byte")]
                assert(bit == word_bit(word, count - 1 - index));
                index += 1;
                assert(reader@ =~= packed.skip(prefix.len() + 8 + index));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;
    use crate::tools::testutil::sample_usize;

    #[test]
    fn test_bitrw() {
        let mut rng = Xorshift::default();
        for n in sample_usize(&mut rng, 16, 0..=256, 1000) {
            let mut writer = BitWriter::default();
            let mut bits = Vec::new();
            for i in 0..n {
                let len = if i < 64 {
                    i as u32 + 1
                } else {
                    rng.random(1..=64)
                };
                for value in [0, u64::MAX >> (64 - len), rng.rand64() >> (64 - len)] {
                    match len {
                        1 => writer.push_bit(value != 0),
                        8 => writer.push_u8(value as u8),
                        _ => writer.push_u64(value, len),
                    }
                    bits.extend((0..len).rev().map(|i| value >> i & 1 != 0));
                }
            }
            let bytes = writer.into_inner();
            let mut reader = BitReader::new(&bytes);
            for &bit in &bits {
                assert_eq!(reader.read_bit(), bit);
            }
            let mut reader = BitReader::new(&bytes);
            for chunk in bits.as_chunks::<8>().0 {
                assert_eq!(
                    reader.read_u8(),
                    chunk.iter().fold(0, |a, &b| (a << 1) | b as u8)
                );
            }
        }
    }

    #[test]
    fn test_serde_unicode() {
        let mut bytes = Vec::new();
        let mut text = String::new();
        for value in (0..=0x10ffff).filter_map(char::from_u32) {
            bytes.clear();
            value.serialize(&mut bytes);
            assert_eq!(bytes, (value as u32).to_le_bytes());
            assert_eq!(char::deserialize(&mut bytes.iter().copied()), value);
            text.push(value);
        }
        for value in [String::new(), text] {
            bytes.clear();
            value.serialize(&mut bytes);
            assert_eq!(&bytes[..size_of::<usize>()], value.len().to_le_bytes());
            assert_eq!(&bytes[size_of::<usize>()..], value.as_bytes());
            assert_eq!(String::deserialize(&mut bytes.iter().copied()), value);
        }
    }

    #[test]
    fn test_serde_string_iterator() {
        for n in 0usize..=8 {
            for mask in 0..1 << n {
                let mut input = n
                    .to_le_bytes()
                    .into_iter()
                    .map(Some)
                    .chain((0..n).map(|i| (mask >> i & 1 != 0).then_some(b'a' + i as u8)))
                    .chain([Some(b'!')]);
                let mut iter = std::iter::from_fn(|| input.next().flatten());
                let expected: String = (0..n)
                    .map(|i| {
                        if mask >> i & 1 != 0 {
                            (b'a' + i as u8) as char
                        } else {
                            '\0'
                        }
                    })
                    .collect();
                assert_eq!(String::deserialize(&mut iter), expected);
                assert_eq!(iter.next(), Some(b'!'));
            }
        }
        let mut iter = 3usize.to_le_bytes().into_iter().chain(*b"a");
        assert_eq!(String::deserialize(&mut iter), "a\0\0");
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_serde() {
        let mut rng = Xorshift::default();
        for n in sample_usize(&mut rng, 16, 0..=256, 1000) {
            let text: String = (0..n as u32)
                .filter_map(char::from_u32)
                .chain((0..n).filter_map(|_| char::from_u32(rng.random(0..=0x10ffff))))
                .collect();
            let value = (
                (0..n)
                    .map(|i| i as u8)
                    .chain(rng.random_iter(..).take(n))
                    .collect::<Vec<u8>>(),
                text,
                (0..n as u64)
                    .chain(rng.random_iter(..).take(n))
                    .collect::<VecDeque<u64>>(),
                rng.random_iter(0..=100)
                    .take(n)
                    .collect::<BTreeSet<usize>>(),
                rng.random_iter(-100..=100)
                    .take(n)
                    .collect::<HashSet<i128>>(),
            );
            let encoded = value.serialize_bytestr();
            let decoded = SerdeByteStr::deserialize_from_bytes(&unescape(encoded.as_bytes()));
            assert_eq!(value, decoded);
        }
        for value in (-4i64..=4)
            .flat_map(|a| (0u64..=8).map(move |b| (a, b)))
            .chain(rng.random_iter((.., ..)).take(1000))
        {
            let encoded = value.serialize_bytestr();
            let decoded = SerdeByteStr::deserialize_from_bytes(&unescape(encoded.as_bytes()));
            assert_eq!(value, decoded);
        }
    }
}
