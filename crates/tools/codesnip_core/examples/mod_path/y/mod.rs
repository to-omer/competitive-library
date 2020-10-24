//! y/mod.rs

mod ya;
mod yb;
#[path = "yc_path.rs"]
mod yc;
mod yd {
    mod yda;
    mod ydb;
    #[path = "ydc_path.rs"]
    mod ydc;
}
#[path = "ye_path"]
mod ye {
    mod yea;
    mod yeb;
    #[path = "yec_path.rs"]
    mod yec;
}
