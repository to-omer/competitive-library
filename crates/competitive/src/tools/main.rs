#![allow(dead_code)]

#[codesnip::skip]
use crate::{
    prepare_io, scan,
    tools::{read_stdin_all_unchecked, Scanner},
};

fn main() {
    #![allow(unused_imports, unused_macros)]
    prepare_io!(_in_buf, scanner, _out);
    macro_rules! print {
        ($($arg:tt)*) => (::std::write!(_out, $($arg)*).expect("io error"))
    }
    macro_rules! println {
        ($($arg:tt)*) => (::std::writeln!(_out, $($arg)*).expect("io error"))
    }
    scan!(scanner, _n);
}

#[macro_export]
macro_rules! prepare_io {
    ($in_buf:ident, $scanner:ident, $out:ident) => {
        use std::io::{stdout, BufWriter, Write as _};
        let $in_buf = read_stdin_all_unchecked();
        let mut $scanner = Scanner::new(&$in_buf);
        let $out = stdout();
        let $out = &mut BufWriter::new($out.lock());
    };
}
