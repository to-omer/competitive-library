#[derive(Clone, Debug)]
struct BucketQueue8 {
    counts: [u32; 1 << 8],
    occupied: [u64; 1 << 2],
    summary: u8,
    maximum: u8,
    len: usize,
}

impl BucketQueue8 {
    fn new() -> Self {
        Self {
            counts: [0; 1 << 8],
            occupied: [0; 1 << 2],
            summary: 0,
            maximum: 0,
            len: 0,
        }
    }

    #[inline]
    fn push(&mut self, value: u8) {
        assert!(self.len < u32::MAX as usize);
        let value = value as usize;
        if self.len == 0 || value > self.maximum as usize {
            self.maximum = value as u8;
        }
        if self.counts[value] == 0 {
            self.occupied[value / 64] |= 1 << (value % 64);
            self.summary |= 1 << (value / 64);
        }
        self.counts[value] += 1;
        self.len += 1;
    }

    fn from_values(values: impl IntoIterator<Item = u8>, len: usize) -> Self {
        assert!(len <= u32::MAX as usize);
        let mut result = Self::new();
        result.len = len;
        for value in values {
            result.counts[value as usize] += 1;
        }
        for (value, &count) in result.counts.iter().enumerate() {
            if count != 0 {
                result.occupied[value / 64] |= 1 << (value % 64);
            }
        }
        for (word, &occupied) in result.occupied.iter().enumerate() {
            if occupied != 0 {
                result.summary |= 1 << word;
            }
        }
        if len != 0 {
            let word = (u8::BITS - 1 - result.summary.leading_zeros()) as usize;
            result.maximum =
                (word * 64 + 63 - result.occupied[word].leading_zeros() as usize) as u8;
        }
        result
    }

    #[inline]
    fn pop(&mut self) -> Option<u8> {
        if self.len == 0 {
            return None;
        }
        let value = self.maximum as usize;
        self.counts[value] -= 1;
        self.len -= 1;
        if self.counts[value] == 0 {
            let word = value / 64;
            self.occupied[word] &= !(1 << (value % 64));
            if self.occupied[word] == 0 {
                self.summary &= !(1 << word);
            }
            if self.len != 0 {
                let word = (u8::BITS - 1 - self.summary.leading_zeros()) as usize;
                self.maximum =
                    (word * 64 + 63 - self.occupied[word].leading_zeros() as usize) as u8;
            }
        }
        Some(value as u8)
    }

    #[inline]
    fn replace(&mut self, value: u8) -> Option<u8> {
        if self.len == 0 {
            self.push(value);
            return None;
        }
        let result = self.maximum;
        if value == result {
            return Some(result);
        }

        let old = result as usize;
        self.counts[old] -= 1;
        if self.counts[old] == 0 {
            let word = old / 64;
            self.occupied[word] &= !(1 << (old % 64));
            if self.occupied[word] == 0 {
                self.summary &= !(1 << word);
            }
        }

        let new = value as usize;
        if self.counts[new] == 0 {
            self.occupied[new / 64] |= 1 << (new % 64);
            self.summary |= 1 << (new / 64);
        }
        self.counts[new] += 1;

        if value > result || self.counts[old] == 0 {
            let word = (u8::BITS - 1 - self.summary.leading_zeros()) as usize;
            self.maximum = (word * 64 + 63 - self.occupied[word].leading_zeros() as usize) as u8;
        }
        Some(result)
    }

    fn clear(&mut self) {
        self.counts.fill(0);
        self.occupied.fill(0);
        self.summary = 0;
        self.maximum = 0;
        self.len = 0;
    }
}

#[derive(Clone, Debug)]
struct BucketQueue16 {
    counts: Vec<u32>,
    occupied: Vec<u64>,
    summary: [u64; 1 << 4],
    top: u16,
    maximum: u16,
    len: usize,
}

impl BucketQueue16 {
    fn new() -> Self {
        Self {
            counts: vec![0; 1 << 16],
            occupied: vec![0; 1 << 10],
            summary: [0; 1 << 4],
            top: 0,
            maximum: 0,
            len: 0,
        }
    }

    #[inline]
    fn push(&mut self, value: u16) {
        assert!(self.len < u32::MAX as usize);
        let value = value as usize;
        if self.len == 0 || value > self.maximum as usize {
            self.maximum = value as u16;
        }
        if self.counts[value] == 0 {
            let word = value / 64;
            self.occupied[word] |= 1 << (value % 64);
            self.summary[word / 64] |= 1 << (word % 64);
            self.top |= 1 << (word / 64);
        }
        self.counts[value] += 1;
        self.len += 1;
    }

    fn from_values(values: impl IntoIterator<Item = u16>, len: usize) -> Self {
        assert!(len <= u32::MAX as usize);
        let mut result = Self::new();
        result.len = len;
        for value in values {
            result.counts[value as usize] += 1;
        }
        for (value, &count) in result.counts.iter().enumerate() {
            if count != 0 {
                result.occupied[value / 64] |= 1 << (value % 64);
            }
        }
        for (word, &occupied) in result.occupied.iter().enumerate() {
            if occupied != 0 {
                result.summary[word / 64] |= 1 << (word % 64);
            }
        }
        for (word, &summary) in result.summary.iter().enumerate() {
            if summary != 0 {
                result.top |= 1 << word;
            }
        }
        if len != 0 {
            let summary = (u16::BITS - 1 - result.top.leading_zeros()) as usize;
            let word = summary * 64 + 63 - result.summary[summary].leading_zeros() as usize;
            result.maximum =
                (word * 64 + 63 - result.occupied[word].leading_zeros() as usize) as u16;
        }
        result
    }

    #[inline]
    fn pop(&mut self) -> Option<u16> {
        if self.len == 0 {
            return None;
        }
        let value = self.maximum as usize;
        self.counts[value] -= 1;
        self.len -= 1;
        if self.counts[value] == 0 {
            let word = value / 64;
            let summary = word / 64;
            self.occupied[word] &= !(1 << (value % 64));
            if self.occupied[word] == 0 {
                self.summary[summary] &= !(1 << (word % 64));
                if self.summary[summary] == 0 {
                    self.top &= !(1 << summary);
                }
            }
            if self.len != 0 {
                let summary = (u16::BITS - 1 - self.top.leading_zeros()) as usize;
                let word = summary * 64 + 63 - self.summary[summary].leading_zeros() as usize;
                self.maximum =
                    (word * 64 + 63 - self.occupied[word].leading_zeros() as usize) as u16;
            }
        }
        Some(value as u16)
    }

    #[inline]
    fn replace(&mut self, value: u16) -> Option<u16> {
        if self.len == 0 {
            self.push(value);
            return None;
        }
        let result = self.maximum;
        if value == result {
            return Some(result);
        }

        let old = result as usize;
        self.counts[old] -= 1;
        if self.counts[old] == 0 {
            let word = old / 64;
            let summary = word / 64;
            self.occupied[word] &= !(1 << (old % 64));
            if self.occupied[word] == 0 {
                self.summary[summary] &= !(1 << (word % 64));
                if self.summary[summary] == 0 {
                    self.top &= !(1 << summary);
                }
            }
        }

        let new = value as usize;
        if self.counts[new] == 0 {
            let word = new / 64;
            self.occupied[word] |= 1 << (new % 64);
            self.summary[word / 64] |= 1 << (word % 64);
            self.top |= 1 << (word / 64);
        }
        self.counts[new] += 1;

        if value > result || self.counts[old] == 0 {
            let summary = (u16::BITS - 1 - self.top.leading_zeros()) as usize;
            let word = summary * 64 + 63 - self.summary[summary].leading_zeros() as usize;
            self.maximum = (word * 64 + 63 - self.occupied[word].leading_zeros() as usize) as u16;
        }
        Some(result)
    }

    fn clear(&mut self) {
        self.counts.fill(0);
        self.occupied.fill(0);
        self.summary.fill(0);
        self.top = 0;
        self.maximum = 0;
        self.len = 0;
    }
}

macro_rules! define_bucket_queue {
    ($name:ident, $doc:literal, $value:ty, $repr:ty, $queue:ty, $sign:expr, $bulk_threshold:expr) => {
        #[doc = $doc]
        #[derive(Clone, Debug)]
        pub struct $name {
            queue: $queue,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    queue: <$queue>::new(),
                }
            }

            #[inline]
            pub fn len(&self) -> usize {
                self.queue.len
            }

            #[inline]
            pub fn is_empty(&self) -> bool {
                self.queue.len == 0
            }

            #[inline]
            pub fn peek(&self) -> Option<$value> {
                (self.queue.len != 0).then_some((self.queue.maximum ^ $sign) as $value)
            }

            /// # Panics
            ///
            /// Panics if the queue already contains `u32::MAX` values.
            #[inline]
            pub fn push(&mut self, value: $value) {
                self.queue.push((value as $repr) ^ $sign);
            }

            #[inline]
            pub fn pop(&mut self) -> Option<$value> {
                self.queue.pop().map(|value| ((value ^ $sign) as $value))
            }

            /// Unconditionally replaces the greatest value, or inserts into an empty queue.
            #[inline]
            pub fn replace(&mut self, value: $value) -> Option<$value> {
                self.queue
                    .replace((value as $repr) ^ $sign)
                    .map(|value| (value ^ $sign) as $value)
            }

            pub fn clear(&mut self) {
                self.queue.clear();
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl From<Vec<$value>> for $name {
            fn from(values: Vec<$value>) -> Self {
                if values.len() >= $bulk_threshold {
                    let len = values.len();
                    Self {
                        queue: <$queue>::from_values(
                            values.into_iter().map(|value| (value as $repr) ^ $sign),
                            len,
                        ),
                    }
                } else {
                    let mut queue = Self::new();
                    queue.extend(values);
                    queue
                }
            }
        }

        impl Extend<$value> for $name {
            fn extend<I>(&mut self, iter: I)
            where
                I: IntoIterator<Item = $value>,
            {
                for value in iter {
                    self.push(value);
                }
            }
        }

        impl FromIterator<$value> for $name {
            fn from_iter<I>(iter: I) -> Self
            where
                I: IntoIterator<Item = $value>,
            {
                Self::from(Vec::from_iter(iter))
            }
        }
    };
}

define_bucket_queue!(
    BucketQueueU8,
    "A fixed 8-bit-universe max-priority queue. `BinaryHeap::peek_mut` can be faster for replacements in tiny queues.",
    u8,
    u8,
    BucketQueue8,
    0,
    1 << 12
);
define_bucket_queue!(
    BucketQueueI8,
    "A fixed 8-bit-universe max-priority queue. `BinaryHeap::peek_mut` can be faster for replacements in tiny queues.",
    i8,
    u8,
    BucketQueue8,
    1 << 7,
    1 << 12
);
define_bucket_queue!(
    BucketQueueU16,
    "A fixed 16-bit-universe max-priority queue that allocates about 264 KiB when empty. `BinaryHeap` can be faster for small queues.",
    u16,
    u16,
    BucketQueue16,
    0,
    1 << 16
);
define_bucket_queue!(
    BucketQueueI16,
    "A fixed 16-bit-universe max-priority queue that allocates about 264 KiB when empty. `BinaryHeap` can be faster for small queues.",
    i16,
    u16,
    BucketQueue16,
    1 << 15,
    1 << 16
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;
    use std::collections::BinaryHeap;

    #[test]
    fn test_bucket_queue() {
        macro_rules! test_queue {
            ($queue:ty, $value:ty) => {{
                let mut rng = Xorshift::default();
                let values: Vec<$value> = (0..10_000).map(|_| rng.rand64() as $value).collect();
                let mut actual: $queue = values.clone().into();
                let mut expected = BinaryHeap::from(values);
                for _ in 0..20_000 {
                    match rng.rand(4) {
                        0 => {
                            let value = rng.rand64() as $value;
                            actual.push(value);
                            expected.push(value);
                        }
                        1 => assert_eq!(actual.pop(), expected.pop()),
                        _ => {
                            let value = rng.rand64() as $value;
                            let old = expected.pop();
                            expected.push(value);
                            assert_eq!(actual.replace(value), old);
                        }
                    }
                    assert_eq!(actual.peek(), expected.peek().copied());
                    assert_eq!(actual.len(), expected.len());
                    assert_eq!(actual.is_empty(), expected.is_empty());
                }
                while let Some(value) = expected.pop() {
                    assert_eq!(actual.pop(), Some(value));
                }
                assert_eq!(actual.pop(), None);
                actual.extend([<$value>::MIN, 0, <$value>::MAX, <$value>::MAX]);
                assert_eq!(actual.pop(), Some(<$value>::MAX));
                assert_eq!(actual.pop(), Some(<$value>::MAX));
                actual.clear();
                assert!(actual.is_empty());
                actual.push(<$value>::MIN);
                assert_eq!(actual.pop(), Some(<$value>::MIN));
            }};
        }

        test_queue!(BucketQueueU8, u8);
        test_queue!(BucketQueueI8, i8);
        test_queue!(BucketQueueU16, u16);
        test_queue!(BucketQueueI16, i16);

        let values = [0_u16, 63, 64, 4095, 4096, u16::MAX, u16::MAX];
        let mut actual = BucketQueueU16::from(values.to_vec());
        let mut expected = BinaryHeap::from(values);
        while !expected.is_empty() {
            assert_eq!(actual.pop(), expected.pop());
        }

        let values: Vec<_> = (0..1 << 16).map(|value| value as u16).collect();
        let mut actual = BucketQueueU16::from(values.clone());
        let mut expected = BinaryHeap::from(values);
        while !expected.is_empty() {
            assert_eq!(actual.pop(), expected.pop());
        }
    }
}
