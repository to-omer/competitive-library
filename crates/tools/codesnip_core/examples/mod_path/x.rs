//! x.rs

mod xa;
mod xb;
#[path = "xc_path.rs"]
mod xc;
mod xd {
    mod xda;
    mod xdb;
    #[path = "xdc_path.rs"]
    mod xdc;
}
#[path = "xe_path"]
mod xe {
    mod xea;
    mod xeb;
    #[path = "xec_path.rs"]
    mod xec;
}
