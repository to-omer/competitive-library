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
                match self {
                    '0'..='9' => Some((self as u8 - b'0') as $ty),
                    _ => None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unicode_conversions() {
        for character in (0..=char::MAX as u32).filter_map(char::from_u32) {
            let digit = character
                .is_ascii_digit()
                .then(|| character as u32 - '0' as u32);
            let lower = character
                .is_ascii_lowercase()
                .then(|| character as u32 - 'a' as u32);
            let upper = character
                .is_ascii_uppercase()
                .then(|| character as u32 - 'A' as u32);
            assert_eq!(character.into_number(), digit);
            assert_eq!(character.into_lower_alphabetic_number(), lower);
            assert_eq!(character.into_upper_alphabetic_number(), upper);
            assert_eq!(character.into_alphabetic_number(), lower.or(upper));
        }
    }

    #[test]
    fn test_ascii_roundtrips() {
        macro_rules! check_type {
            ($ty:ty) => {
                for number in (0..=u8::MAX).map(|x| x as $ty).chain([<$ty>::MAX]) {
                    let digit = <char as CharConvertTryFrom<$ty>>::from_number(number);
                    let lower =
                        <char as CharConvertTryFrom<$ty>>::from_lower_alphabetic_number(number);
                    let upper =
                        <char as CharConvertTryFrom<$ty>>::from_upper_alphabetic_number(number);
                    assert_eq!(digit.is_some(), number < 10);
                    assert_eq!(lower.is_some(), number < 26);
                    assert_eq!(upper.is_some(), number < 26);
                    assert_eq!(
                        <char as CharConvertTryFrom<$ty>>::from_alphabetic_number(number),
                        upper
                    );
                    assert_eq!(
                        <u8 as CharConvertTryFrom<$ty>>::from_number(number),
                        digit.map(|c| c as u8)
                    );
                    assert_eq!(
                        <u8 as CharConvertTryFrom<$ty>>::from_lower_alphabetic_number(number),
                        lower.map(|c| c as u8)
                    );
                    assert_eq!(
                        <u8 as CharConvertTryFrom<$ty>>::from_upper_alphabetic_number(number),
                        upper.map(|c| c as u8)
                    );
                    assert_eq!(
                        <u8 as CharConvertTryFrom<$ty>>::from_alphabetic_number(number),
                        upper.map(|c| c as u8)
                    );
                    if let Some(digit) = digit {
                        assert_eq!(
                            <char as CharConvertTryInto<$ty>>::into_number(digit),
                            Some(number)
                        );
                        assert_eq!(
                            <u8 as CharConvertTryInto<$ty>>::into_number(digit as u8),
                            Some(number)
                        );
                    }
                    if let (Some(lower), Some(upper)) = (lower, upper) {
                        assert_eq!(
                            <char as CharConvertTryInto<$ty>>::into_lower_alphabetic_number(lower),
                            Some(number)
                        );
                        assert_eq!(
                            <char as CharConvertTryInto<$ty>>::into_upper_alphabetic_number(upper),
                            Some(number)
                        );
                        assert_eq!(
                            <char as CharConvertTryInto<$ty>>::into_alphabetic_number(lower),
                            Some(number)
                        );
                        assert_eq!(
                            <char as CharConvertTryInto<$ty>>::into_alphabetic_number(upper),
                            Some(number)
                        );
                        assert_eq!(
                            <u8 as CharConvertTryInto<$ty>>::into_lower_alphabetic_number(
                                lower as u8
                            ),
                            Some(number)
                        );
                        assert_eq!(
                            <u8 as CharConvertTryInto<$ty>>::into_upper_alphabetic_number(
                                upper as u8
                            ),
                            Some(number)
                        );
                        assert_eq!(
                            <u8 as CharConvertTryInto<$ty>>::into_alphabetic_number(lower as u8),
                            Some(number)
                        );
                        assert_eq!(
                            <u8 as CharConvertTryInto<$ty>>::into_alphabetic_number(upper as u8),
                            Some(number)
                        );
                    }
                }
            };
        }
        check_type!(u8);
        check_type!(u16);
        check_type!(u32);
        check_type!(u64);
        check_type!(u128);
        check_type!(usize);
    }
}
