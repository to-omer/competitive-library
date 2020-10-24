//! lib.rs

mod a;
mod b;
#[path = "c_path.rs"]
mod c;
mod d {
    mod da;
    mod db;
    #[path = "dc_path.rs"]
    mod dc;
}
#[path = "e_path"]
#[rustfmt::skip]
mod e {
    mod ea;
    mod eb;
    #[path = "ec_path.rs"]
    mod ec;
}

#[rustfmt::skip]
mod x;
#[rustfmt::skip]
mod y;
#[rustfmt::skip]
#[path = "z_path.rs"]
mod z;
