use competitive::tools::Scanner;

#[cargo_snippet::snippet]
#[cargo_snippet::snippet(include = "scanner")]
#[cargo_snippet::snippet(include = "zero_one")]
#[cargo_snippet::snippet(include = "minmax")]
fn main() {
    #[allow(unused_imports)]
    use std::io::{Read as _, Write as _};
    let __out = std::io::stdout();
    let mut __in_buf = String::new();
    std::io::stdin().read_to_string(&mut __in_buf).unwrap();
    let mut scanner = Scanner::new(&__in_buf);
    #[allow(unused_macros)]
    macro_rules! scan {
        () => {
            scan!(usize)
        };
        (($($t:tt),*)) => {
            ($(scan!($t)),*)
        };
        ([$t:tt; $len:expr]) => {
            (0..$len).map(|_| scan!($t)).collect::<Vec<_>>()
        };
        ({ chars: $b:expr }) => {
            scanner.scan_chars_with($b)
        };
        ({ $t:tt => $f:expr }) => {
            $f(scan!($t))
        };
        (chars) => {
            scanner.scan_chars()
        };
        ($t:ty) => {
            scanner.scan::<$t>()
        };
    }
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
            let mut iter = $iter.into_iter();
            if let Some(item) = iter.next() {
                print!("{}", item);
            }
            for item in iter {
                print!("{}{}", $sep, item);
            }
            println!();
        };
    }
    let _n = scan!();
}
