#![allow(dead_code)]
#![allow(clippy::crate_in_macro_def)]

#[codesnip::skip]
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
    /// - `prepare!();`: default (all input scanner (`sc!`, `sv!`) + buf print (`pp!`, `dg!`))
    /// - `prepare!(?);`: interactive (line scanner (`scln!`) + buf print (`pp!`, `dg!`))
    #[macro_export]
    macro_rules! prepare {
        (@output ($dol:tt)) => {
            #[allow(unused_imports)]
            use std::io::Write as _;
            let __out = std::io::stdout();
            #[allow(unused_mut,unused_variables)]
            let mut __out = std::io::BufWriter::new(__out.lock());
            #[allow(unused_macros)]
            /// [`iter_print!`] for buffered stdout.
            macro_rules! pp { ($dol($dol t:tt)*) => { $dol crate::iter_print!(__out, $dol($dol t)*) } }
            #[cfg(debug_assertions)]
            #[allow(unused_macros)]
            /// [`iter_print!`] for buffered stderr. Do nothing in release mode.
            macro_rules! dg {
                ($dol($dol t:tt)*) => {{
                    #[allow(unused_imports)]
                    use std::io::Write as _;
                    let __err = std::io::stderr();
                    #[allow(unused_mut,unused_variables)]
                    let mut __err = std::io::BufWriter::new(__err.lock());
                    $dol crate::iter_print!(__err, $dol($dol t)*);
                    let _ = __err.flush();
                }}
            }
            #[cfg(not(debug_assertions))]
            #[allow(unused_macros)]
            /// [`iter_print!`] for buffered stderr. Do nothing in release mode.
            macro_rules! dg { ($dol($dol t:tt)*) => {} }
        };
        (@normal ($dol:tt)) => {
            let __in_buf = read_stdin_all_unchecked();
            #[allow(unused_mut,unused_variables)]
            let mut __scanner = Scanner::new(&__in_buf);
            #[allow(unused_macros)]
            macro_rules! sc { ($dol($dol t:tt)*) => { $dol crate::scan!(__scanner, $dol($dol t)*) } }
            #[allow(unused_macros)]
            macro_rules! sv { ($dol($dol t:tt)*) => { $dol crate::scan_value!(__scanner, $dol($dol t)*) } }
        };
        (@interactive ($dol:tt)) => {
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
        () => { $crate::prepare!(@output ($)); $crate::prepare!(@normal ($)) };
        (?) => { $crate::prepare!(@output ($)); $crate::prepare!(@interactive ($)) };
    }
}
