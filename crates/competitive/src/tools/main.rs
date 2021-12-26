#![allow(dead_code)]

#[cfg_attr(nightly, codesnip::skip)]
use crate::tools::{read_stdin_all_unchecked, Scanner};

#[cfg_attr(any(), rust_minify::skip)]
pub fn main() {
    crate::prepare!();
    sc!(_n);
}

#[allow(unused_imports)]
use std::{
    cmp::{Ordering, Reverse},
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, VecDeque},
};

mod main_macros {
    /// Prepare useful macros.
    /// - `prepare!();`: default (all input scanner (`sc!`, `sv!`) + buf print (`pp!`))
    /// - `prepare!(?);`: interactive (line scanner (`scln!`) + buf print (`pp!`))
    #[macro_export]
    macro_rules! prepare {
        (@normal ($dol:tt)) => {
            #[allow(unused_imports)]
            use std::io::Write as _;
            let __out = std::io::stdout();
            #[allow(unused_mut,unused_variables)]
            let mut __out = std::io::BufWriter::new(__out.lock());
            #[allow(unused_macros)]
            macro_rules! pp { ($dol($dol t:tt)*) => { $dol crate::iter_print!(__out, $dol($dol t)*) } }
            let __in_buf = read_stdin_all_unchecked();
            #[allow(unused_mut,unused_variables)]
            let mut __scanner = Scanner::new(&__in_buf);
            #[allow(unused_macros)]
            macro_rules! sc { ($dol($dol t:tt)*) => { $dol crate::scan!(__scanner, $dol($dol t)*) } }
            #[allow(unused_macros)]
            macro_rules! sv { ($dol($dol t:tt)*) => { $dol crate::scan_value!(__scanner, $dol($dol t)*) } }
        };
        (@interactive ($dol:tt)) => {
            #[allow(unused_imports)]
            use std::io::Write as _;
            let __out = std::io::stdout();
            #[allow(unused_mut,unused_variables)]
            let mut __out = std::io::BufWriter::new(__out.lock());
            #[allow(unused_macros)]
            /// - to flush: `pp!(@flush);`
            macro_rules! pp { ($dol($dol t:tt)*) => { $dol crate::iter_print!(__out, $dol($dol t)*) } }
            #[allow(unused_macros)]
            /// Scan a line, and previous line will be truncated in the next call.
            macro_rules! scln {
                ($dol($dol t:tt)*) => {
                    let __in_buf = read_stdin_line();
                    #[allow(unused_mut,unused_variables)]
                    let mut __scanner = Scanner::new(&__in_buf);
                    $dol crate::scan!(__scanner, $dol($dol t)*)
                }
            }
        };
        () => { $crate::prepare!(@normal ($)) };
        (?) => { $crate::prepare!(@interactive ($)) };
    }
}
