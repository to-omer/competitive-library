//! z_path.rs

mod za;
mod zb;
#[path = "zc_path.rs"]
mod zc;
mod zd {
    mod zda;
    mod zdb;
    #[path = "zdc_path.rs"]
    mod zdc;
}
#[path = "ze_path"]
mod ze {
    mod zea;
    mod zeb;
    #[path = "zec_path.rs"]
    mod zec;
}
