macro_rules! input {
    ($t:tt) => {};
}

#[cargo_snippet::snippet]
#[cargo_snippet::snippet(include = "input")]
fn main() {
    #[allow(unused_imports)]
    use std::io::Write as _;
    let __out = std::io::stdout();
    #[allow(unused_mut, unused_variables)]
    let mut __out = std::io::BufWriter::new(__out.lock());
    #[allow(unused_macros)]
    macro_rules! print {
        ($($arg:tt)*) => (::std::write!(__out, $($arg)*).unwrap())
    }
    #[allow(unused_macros)]
    macro_rules! println {
        ($($arg:tt)*) => (::std::writeln!(__out, $($arg)*).unwrap())
    }
    #[allow(unused_macros)]
    macro_rules! echo {
        ($iter:expr) => {
            echo!($iter, "\n")
        };
        ($iter:expr, $sep:expr) => {
            let mut iter = $iter;
            if let Some(item) = iter.next() {
                print!("{}", item);
            }
            for item in iter {
                print!("{}{}", $sep, item);
            }
            println!();
        };
    }
    input!(n);
}
