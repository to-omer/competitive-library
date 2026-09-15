pub use crate::tools::{
    Bytes, Chars, FastInput, FastIterPrint, FastOutput, FastPrint, IterPrint, Scan, ScanSource,
    Scanner, Usize1, read_all_unchecked,
};
pub use crate::{iter_print, prepare_io, scan};
pub use std::io::{Read, Write};

/// Prepare `sc!`, `sv!`, `pp!`, and `dg!` with fast I/O for the supplied reader and writer.
/// Every read must satisfy [`FastInput`]'s token requirements.
#[macro_export]
macro_rules! prepare_io {
    (@inner ($dol:tt) $reader:expr, $writer:expr) => {
        #[allow(unused_imports)]
        use $crate::tools::{FastInput, FastIterPrint, FastOutput, FastPrint, IterPrint, ScanSource as _, read_all_unchecked};
        let mut __in_buf = read_all_unchecked($reader);
        __in_buf.push_str("                 ");
        #[allow(unused_mut, unused_variables)]
        let mut __scanner = unsafe { FastInput::from_slice(__in_buf.as_bytes()) };
        #[allow(unused_mut, unused_variables)]
        let mut __out = FastOutput::new($writer);
        #[allow(unused_macros)]
        macro_rules! sc { ($dol($dol t:tt)*) => { $crate::scan!(__scanner, $dol($dol t)*) } }
        #[allow(unused_macros)]
        macro_rules! sv { ($dol($dol t:tt)*) => { $crate::scan_value!(__scanner, $dol($dol t)*) } }
        #[allow(unused_macros)]
        macro_rules! pp { ($dol($dol t:tt)*) => { $crate::iter_print!(fast; __out, $dol($dol t)*) } }
        $crate::prepare!(@debug ($));
    };
    ($reader:expr, $writer:expr $(,)?) => {
        $crate::prepare_io!(@inner ($) $reader, $writer);
    };
}
