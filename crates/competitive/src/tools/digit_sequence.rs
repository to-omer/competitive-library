pub trait ToDigitSequence: Sized {
    fn to_digit_sequence(&self) -> Vec<Self>;
    fn to_digit_sequence_radix(&self, radix: Self) -> Vec<Self>;
    fn to_digit_sequence_len(&self, len: usize) -> Vec<Self>;
    fn to_digit_sequence_radix_len(&self, radix: Self, len: usize) -> Vec<Self>;
}

macro_rules! impl_to_digit_sequence {
    ($($t:ty)*) => {
        $(impl ToDigitSequence for $t {
            fn to_digit_sequence(&self) -> Vec<$t> {
                self.to_digit_sequence_radix(10)
            }
            fn to_digit_sequence_radix(&self, radix: Self) -> Vec<$t> {
                let mut x = *self;
                let mut res: Vec<$t> = vec![];
                while x > 0 {
                    res.push(x % radix);
                    x /= radix;
                }
                res.reverse();
                res
            }
            fn to_digit_sequence_len(&self, len: usize) -> Vec<$t> {
                self.to_digit_sequence_radix_len(10, len)
            }
            fn to_digit_sequence_radix_len(&self, radix: Self, len: usize) -> Vec<$t> {
                let mut x = *self;
                let mut res: Vec<$t> = vec![0; len];
                for r in res.iter_mut().rev() {
                    if x == 0 {
                        break;
                    }
                    *r = x % radix;
                    x /= radix;
                }
                res
            }
        })*
    };
}
impl_to_digit_sequence!(u8 u16 u32 u64 u128 usize);
