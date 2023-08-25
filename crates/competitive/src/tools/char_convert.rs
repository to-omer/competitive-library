pub trait CharConvertTryInto<T>: Sized {
    /// map `'0'..='9'` to `0..=9`
    fn into_number(self) -> Option<T>;

    /// map (`'A'..='Z'` or `'a'..='z'`) to `0..=25`
    fn into_alphabetic_number(self) -> Option<T>;

    /// map `'a'..='z'` to `0..=25`
    fn into_lower_alphabetic_number(self) -> Option<T>;

    /// map `'A'..='Z'` to `0..=25`
    fn into_upper_alphabetic_number(self) -> Option<T>;
}

pub trait CharConvertTryFrom<T>: Sized {
    /// map `0..=9` to `'0'..='9'`
    fn from_number(num: T) -> Option<Self>;

    /// map `0..=25` to `'A'..='Z'`
    fn from_alphabetic_number(num: T) -> Option<Self>;

    /// map `0..=25` to `'a'..='z'`
    fn from_lower_alphabetic_number(num: T) -> Option<Self>;

    /// map `0..=25` to `'A'..='Z'`
    fn from_upper_alphabetic_number(num: T) -> Option<Self>;
}

macro_rules! impl_char_convert_unsigned {
    ($($ty:ident)*) => {$(
        impl CharConvertTryInto<$ty> for char {
            fn into_number(self) -> Option<$ty> {
                let num = (self as u8).wrapping_sub(b'0');
                if num < 10 {
                    Some(num as $ty)
                } else {
                    None
                }
            }

            fn into_alphabetic_number(self) -> Option<$ty> {
                match self {
                    'A'..='Z' => Some((self as u8 - b'A') as $ty),
                    'a'..='z' => Some((self as u8 - b'a') as $ty),
                    _ => None,
                }
            }

            fn into_lower_alphabetic_number(self) -> Option<$ty> {
                if self.is_ascii_lowercase() {
                    Some((self as u8 - b'a') as $ty)
                } else {
                    None
                }
            }

            fn into_upper_alphabetic_number(self) -> Option<$ty> {
                if self.is_ascii_uppercase() {
                    Some((self as u8 - b'A') as $ty)
                } else {
                    None
                }
            }
        }

        impl CharConvertTryFrom<$ty> for char {
            fn from_number(num: $ty) -> Option<Self> {
                if num < 10 {
                    Some((b'0' + num as u8) as char)
                } else {
                    None
                }
            }

            fn from_alphabetic_number(num: $ty) -> Option<Self> {
                if num < 26 {
                    Some((b'A' + num as u8) as char)
                } else {
                    None
                }
            }

            fn from_lower_alphabetic_number(num: $ty) -> Option<Self> {
                if num < 26 {
                    Some((b'a' + num as u8) as char)
                } else {
                    None
                }
            }

            fn from_upper_alphabetic_number(num: $ty) -> Option<Self> {
                if num < 26 {
                    Some((b'A' + num as u8) as char)
                } else {
                    None
                }
            }
        }

        impl CharConvertTryInto<$ty> for u8 {
            fn into_number(self) -> Option<$ty> {
                let num = self.wrapping_sub(b'0');
                if num < 10 {
                    Some(num as $ty)
                } else {
                    None
                }
            }

            fn into_alphabetic_number(self) -> Option<$ty> {
                match self {
                    b'A'..=b'Z' => Some((self - b'A') as $ty),
                    b'a'..=b'z' => Some((self - b'a') as $ty),
                    _ => None,
                }
            }

            fn into_lower_alphabetic_number(self) -> Option<$ty> {
                if self.is_ascii_lowercase() {
                    Some((self - b'a') as $ty)
                } else {
                    None
                }
            }

            fn into_upper_alphabetic_number(self) -> Option<$ty> {
                if self.is_ascii_uppercase() {
                    Some((self - b'A') as $ty)
                } else {
                    None
                }
            }
        }

        impl CharConvertTryFrom<$ty> for u8 {
            fn from_number(num: $ty) -> Option<Self> {
                if num < 10 {
                    Some(b'0' + num as u8)
                } else {
                    None
                }
            }

            fn from_alphabetic_number(num: $ty) -> Option<Self> {
                if num < 26 {
                    Some(b'A' + num as u8)
                } else {
                    None
                }
            }

            fn from_lower_alphabetic_number(num: $ty) -> Option<Self> {
                if num < 26 {
                    Some(b'a' + num as u8)
                } else {
                    None
                }
            }

            fn from_upper_alphabetic_number(num: $ty) -> Option<Self> {
                if num < 26 {
                    Some(b'A' + num as u8)
                } else {
                    None
                }
            }
        }
    )*};
}
impl_char_convert_unsigned!(u8 u16 u32 u64 u128 usize);
