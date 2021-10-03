pub trait CharTools: Sized {
    /// map `0..=9` to `'0'..='9'`
    fn from_number(num: u32) -> Option<Self>;
    /// map `'0'..='9'` to `0..=9`
    fn to_number(self) -> Option<u32>;
    /// map `0..=25` to `'A'..='Z'`
    fn from_alphabetic_number(num: u32) -> Option<Self>;
    /// map (`'A'..='Z'` or `'a'..='z'`) to `0..=25`
    fn to_alphabetic_number(self) -> Option<u32>;
    /// map `0..=25` to `'a'..='z'`
    fn from_lower_alphabetic_number(num: u32) -> Option<Self>;
    /// map `'a'..='z'` to `0..=25`
    fn to_lower_alphabetic_number(self) -> Option<u32>;
    /// map `0..=25` to `'A'..='Z'`
    fn from_upper_alphabetic_number(num: u32) -> Option<Self>;
    /// map `'A'..='Z'` to `0..=25`
    fn to_upper_alphabetic_number(self) -> Option<u32>;
}
impl CharTools for char {
    fn from_number(num: u32) -> Option<Self> {
        if num < 10 {
            Some((b'0' + num as u8) as char)
        } else {
            None
        }
    }
    fn to_number(self) -> Option<u32> {
        let num = (self as u32).wrapping_sub('0' as u32);
        if num < 10 {
            Some(num)
        } else {
            None
        }
    }
    fn from_alphabetic_number(num: u32) -> Option<Self> {
        if num < 26 {
            Some((b'A' + num as u8) as char)
        } else {
            None
        }
    }
    fn to_alphabetic_number(self) -> Option<u32> {
        match self {
            'A'..='Z' => Some(self as u32 - 'A' as u32),
            'a'..='z' => Some(self as u32 - 'a' as u32),
            _ => None,
        }
    }
    fn from_lower_alphabetic_number(num: u32) -> Option<Self> {
        if num < 26 {
            Some((b'a' + num as u8) as char)
        } else {
            None
        }
    }
    fn to_lower_alphabetic_number(self) -> Option<u32> {
        if self.is_ascii_lowercase() {
            Some(self as u32 - 'a' as u32)
        } else {
            None
        }
    }
    fn from_upper_alphabetic_number(num: u32) -> Option<Self> {
        if num < 26 {
            Some((b'A' + num as u8) as char)
        } else {
            None
        }
    }
    fn to_upper_alphabetic_number(self) -> Option<u32> {
        if self.is_ascii_uppercase() {
            Some(self as u32 - 'A' as u32)
        } else {
            None
        }
    }
}
impl CharTools for u8 {
    fn from_number(num: u32) -> Option<Self> {
        if num < 10 {
            Some(b'0' + num as u8)
        } else {
            None
        }
    }
    fn to_number(self) -> Option<u32> {
        let num = (self as u32).wrapping_sub(b'0' as u32);
        if num < 10 {
            Some(num)
        } else {
            None
        }
    }
    fn from_alphabetic_number(num: u32) -> Option<Self> {
        if num < 26 {
            Some(b'A' + num as u8)
        } else {
            None
        }
    }
    fn to_alphabetic_number(self) -> Option<u32> {
        match self {
            b'A'..=b'Z' => Some(self as u32 - b'A' as u32),
            b'a'..=b'z' => Some(self as u32 - b'a' as u32),
            _ => None,
        }
    }
    fn from_lower_alphabetic_number(num: u32) -> Option<Self> {
        if num < 26 {
            Some(b'a' + num as u8)
        } else {
            None
        }
    }
    fn to_lower_alphabetic_number(self) -> Option<u32> {
        if self.is_ascii_lowercase() {
            Some(self as u32 - b'a' as u32)
        } else {
            None
        }
    }
    fn from_upper_alphabetic_number(num: u32) -> Option<Self> {
        if num < 26 {
            Some(b'A' + num as u8)
        } else {
            None
        }
    }
    fn to_upper_alphabetic_number(self) -> Option<u32> {
        if self.is_ascii_uppercase() {
            Some(self as u32 - b'A' as u32)
        } else {
            None
        }
    }
}
