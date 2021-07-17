#[macro_export]
macro_rules! add_assign {
    ($l:expr, $r:expr) => {{
        let r = $r;
        $l += r;
    }};
}

#[macro_export]
macro_rules! sub_assign {
    ($l:expr, $r:expr) => {{
        let r = $r;
        $l -= r;
    }};
}

#[macro_export]
macro_rules! mul_assign {
    ($l:expr, $r:expr) => {{
        let r = $r;
        $l *= r;
    }};
}

#[macro_export]
macro_rules! div_assign {
    ($l:expr, $r:expr) => {{
        let r = $r;
        $l /= r;
    }};
}

#[macro_export]
macro_rules! rem_assign {
    ($l:expr, $r:expr) => {{
        let r = $r;
        $l %= r;
    }};
}

#[macro_export]
macro_rules! bitand_assign {
    ($l:expr, $r:expr) => {{
        let r = $r;
        $l &= r;
    }};
}

#[macro_export]
macro_rules! bitor_assign {
    ($l:expr, $r:expr) => {{
        let r = $r;
        $l |= r;
    }};
}

#[macro_export]
macro_rules! bitxor_assign {
    ($l:expr, $r:expr) => {{
        let r = $r;
        $l ^= r;
    }};
}

#[macro_export]
macro_rules! shl_assign {
    ($l:expr, $r:expr) => {{
        let r = $r;
        $l <<= r;
    }};
}

#[macro_export]
macro_rules! shr_assign {
    ($l:expr, $r:expr) => {{
        let r = $r;
        $l >>= r;
    }};
}
