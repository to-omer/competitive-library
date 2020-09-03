use competitive::tools::Scanner;

#[cargo_snippet::snippet]
#[cargo_snippet::snippet(include = "scanner")]
#[cargo_snippet::snippet(include = "zero_one")]
#[cargo_snippet::snippet(include = "minmax")]
fn main() {
    #![allow(unused_imports, unused_macros)]
    use std::io::{stdin, stdout, BufWriter, Read as _, Write as _};
    let mut _in_buf = Vec::new();
    stdin().read_to_end(&mut _in_buf).expect("io error");
    let _in_buf = unsafe { String::from_utf8_unchecked(_in_buf) };
    let mut scanner = Scanner::new(&_in_buf);
    macro_rules! scan {
        () => { scan!(usize) };
        (($($t:tt),*)) => { ($(scan!($t)),*) };
        ([$t:ty; $len:expr]) => { scanner.scan_vec::<$t>($len) };
        ([$t:tt; $len:expr]) => { (0..$len).map(|_| scan!($t)).collect::<Vec<_>>() };
        ([$t:ty]) => { scanner.iter::<$t>() };
        ({ $e:expr }) => { scanner.mscan($e) };
        ($t:ty) => { scanner.scan::<$t>() };
    }
    let _out = stdout();
    let mut _out = BufWriter::new(_out.lock());
    macro_rules! print {
        ($($arg:tt)*) => (::std::write!(_out, $($arg)*).expect("io error"))
    }
    macro_rules! println {
        ($($arg:tt)*) => (::std::writeln!(_out, $($arg)*).expect("io error"))
    }
    macro_rules! echo {
        ($iter:expr) => {
            echo!($iter, '\n')
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
