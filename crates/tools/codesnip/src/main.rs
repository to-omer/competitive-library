use codesnip::Opt;
use std::process::exit;

const EXIT_FAILURE: i32 = 1;

fn main() {
    if let Err(err) = Opt::from_args().execute() {
        eprintln!("error: {}", err);
        exit(EXIT_FAILURE);
    }
}
