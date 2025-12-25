use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::tools::{FastInput, FastOutput};

#[verify::library_checker("many_aplusb")]
pub fn many_aplusb(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, t);
    for (a, b) in scanner.iter::<(usize, usize)>().take(t) {
        writeln!(writer, "{}", a + b).ok();
    }
}

#[verify::library_checker("many_aplusb")]
pub fn many_aplusb_fast(reader: impl Read, writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut writer = FastOutput::with_capacity(1 << 12, writer);
    let mut scanner = unsafe { FastInput::from_slice(s.as_bytes()) };
    let t = unsafe { scanner.u64() };
    for _ in 0..t {
        let a = unsafe { scanner.u64() };
        let b = unsafe { scanner.u64() };
        writer.u64(a + b);
        writer.byte(b'\n');
    }
}
