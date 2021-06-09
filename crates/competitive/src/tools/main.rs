#![allow(dead_code)]

#[codesnip::skip]
use crate::tools::{read_stdin_all_unchecked, Scanner};

#[cfg_attr(any(), rust_minify::skip)]
pub fn main() {
    crate::prepare!();
    sc!(_n);
}

mod main_macros {
    /// - `prepare!();`: default (all scanner + buf print)
    /// - `prepare!(?);`: interactive (line scanner + buf print)
    /// - `prepare!(!);`: line scanner
    #[macro_export]
    macro_rules! prepare {
        (@buf_print ($dol:tt)) => {
            #[allow(unused_imports)]
            use std::io::Write as _;
            let out = std::io::stdout();
            #[allow(unused_mut,unused_variables)]
            let mut out = std::io::BufWriter::new(out.lock());
            #[allow(unused_macros)]
            macro_rules! bprint { ($dol($dol t:tt)*) => { ::std::write!(out, $dol($dol t)*).expect("io error") } }
            #[allow(unused_macros)]
            macro_rules! bprintln { ($dol($dol t:tt)*) => { ::std::writeln!(out, $dol($dol t)*).expect("io error") } }
            #[allow(unused_macros)]
            macro_rules! bflush { () => { out.flush().expect("io error") } }
            #[allow(unused_macros)]
            macro_rules! pp { ($dol($dol t:tt)*) => { $dol crate::iter_print!(out, $dol($dol t)*) } }
        };
        (@normal ($dol:tt)) => {
            let in_buf = read_stdin_all_unchecked();
            #[allow(unused_mut,unused_variables)]
            let mut scanner = Scanner::new(&in_buf);
            #[allow(unused_macros)]
            macro_rules! sc { ($dol($dol t:tt)*) => { $dol crate::scan!(scanner, $dol($dol t)*) } }
            $crate::prepare! { @buf_print ($) }
        };
        (@interactive ($dol:tt)) => {
            let in_buf = read_stdin_line();
            #[allow(unused_mut,unused_variables)]
            let mut scanner = Scanner::new(&in_buf);
            #[allow(unused_macros)]
            macro_rules! sc { ($dol($dol t:tt)*) => { $dol crate::scan!(scanner, $dol($dol t)*) } }
            $crate::prepare! { @buf_print ($) }
        };
        (@line_scanner ($dol:tt)) => {
            let in_buf = read_stdin_line();
            #[allow(unused_mut,unused_variables)]
            let mut scanner = Scanner::new(&in_buf);
            #[allow(unused_macros)]
            macro_rules! sc { ($dol($dol t:tt)*) => { $dol crate::scan!(scanner, $dol($dol t)*) } }
        };
        () => { $crate::prepare!(@normal ($)) };
        (?) => { $crate::prepare!(@interactive ($)) };
        (!) => { $crate::prepare!(@line_scanner ($)) };
    }
}
