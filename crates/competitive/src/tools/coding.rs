use std::{
    char::from_u32_unchecked,
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, VecDeque},
    hash::Hash,
    iter::repeat_with,
    mem::size_of,
};

pub fn unescape(bytes: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(bytes.len());
    let mut escape = false;
    for b in bytes {
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
    }
    buf
}

fn to_bytestring(bytes: &[u8]) -> String {
    let mut count = [0usize; 128];
    let mut c = 0u8;
    let mut w = 0u32;
    for &b in bytes {
        w += 1;
        count[(c | b >> w) as usize] += 1;
        if w == 7 {
            count[(b & 0b1111111u8) as usize] += 1;
            c = 0;
            w = 0;
        } else {
            c = b << (7 - w) & 0b1111111u8;
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
        escape!(c | b >> w);
        if w == 7 {
            escape!(b & 0b1111111u8);
            c = 0;
            w = 0;
        } else {
            c = b << (7 - w) & 0b1111111u8;
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
            buf.push(c | b >> w);
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

#[derive(Debug)]
struct BitWriter {
    bytes: Vec<u8>,
    last: u8,
    w: u32,
}

impl Default for BitWriter {
    fn default() -> Self {
        Self {
            bytes: Default::default(),
            last: Default::default(),
            w: 8,
        }
    }
}

impl BitWriter {
    fn push_bit(&mut self, b: bool) {
        self.w -= 1;
        self.last |= (b as u8) << self.w;
        if self.w == 0 {
            self.bytes.push(self.last);
            self.w = 8;
            self.last = 0;
        }
    }
    fn push_u8(&mut self, b: u8) {
        self.bytes.push(self.last | (b >> (8 - self.w)));
        if self.w < 8 {
            self.last = b << self.w;
        }
    }
    fn push_u64(&mut self, b: u64, mut c: u32) {
        let k = self.w.min(c);
        self.w -= k;
        c -= k;
        self.last |= (b >> c << self.w) as u8;
        if self.w == 0 {
            self.bytes.push(self.last);
            let (s, t) = (c / 8, c % 8);
            for _ in 0..s {
                c -= 8;
                self.bytes.push((b >> c) as u8);
            }
            self.last = 0;
            self.w = 8 - t;
            if t > 0 {
                self.last = (b as u8) << self.w;
            }
        }
    }
    fn into_inner(mut self) -> Vec<u8> {
        if self.w < 8 {
            self.bytes.push(self.last);
        }
        self.bytes
    }
}

#[derive(Debug)]
struct BitReader<'a> {
    bytes: &'a [u8],
    pos: u32,
}

impl<'a> BitReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }
    fn read_bit(&mut self) -> bool {
        let b = self.bytes[0] >> (7 - self.pos) & 1 == 1;
        self.pos += 1;
        if self.pos == 8 {
            self.pos = 0;
            self.bytes = &self.bytes[1..];
        }
        b
    }
    fn read_u8(&mut self) -> u8 {
        let b = self.bytes[0] << self.pos;
        self.bytes = &self.bytes[1..];
        if self.pos == 0 {
            b
        } else {
            b | self.bytes[0] >> (8 - self.pos)
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
                make_table(r, code << 1 | 1, len + 1, table);
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
        Self::deserialize(&mut bytes.as_slice().iter().cloned())
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
        unsafe { from_u32_unchecked(u32::deserialize(iter)) }
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
        unsafe {
            String::from_utf8_unchecked(
                repeat_with(|| u8::deserialize(iter))
                    .take(n)
                    .collect::<Vec<u8>>(),
            )
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitrw() {
        let mut writer = BitWriter::default();
        for i in 0..10 {
            writer.push_bit(i % 3 == 0);
        }
        for i in 0..10 {
            writer.push_u8(i);
        }
        for i in 1..=10 {
            writer.push_u64(i, i as _);
        }
        let bytes = writer.into_inner();

        let mut reader = BitReader::new(&bytes);
        for i in 0..10 {
            assert_eq!(i % 3 == 0, reader.read_bit());
        }
        for i in 0..10 {
            assert_eq!(i, reader.read_u8());
        }
        for i in 1..=10 {
            let mut x = 0u64;
            for j in (0..i).rev() {
                x |= (reader.read_bit() as u64) << j;
            }
            assert_eq!(i, x);
        }
    }

    #[test]
    fn test_serde() {
        let a = (
            (0..=255).collect::<Vec<u8>>(),
            String::from_utf8((0..128).collect::<Vec<u8>>()).unwrap(),
            (0..=255).collect::<VecDeque<u64>>(),
            (0..=255).collect::<BTreeSet<usize>>(),
            (-255..=255).collect::<HashSet<i128>>(),
        );
        let b = a.serialize_bytestr();
        let c = SerdeByteStr::deserialize_from_bytes(&unescape(b.as_bytes()));
        assert_eq!(a, c);

        let a = (0, 0);
        let b = a.serialize_bytestr();
        let c = SerdeByteStr::deserialize_from_bytes(&unescape(b.as_bytes()));
        assert_eq!(a, c);
    }
}
