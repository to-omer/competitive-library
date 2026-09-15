use competitive::prelude::*;
use competitive::tools::{FastInput, FastOutput};

#[verify::library_checker("many_aplusb")]
pub fn many_aplusb(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(t);
    for (a, b) in sv!([(usize, usize)]).take(t) {
        pp!(a + b);
    }
}

#[verify::library_checker("many_aplusb")]
pub fn many_aplusb_fast(reader: impl Read, writer: impl Write) {
    let mut s = read_all_unchecked(reader);
    s.push_str("                ");
    let mut writer = FastOutput::new(writer);
    let mut scanner = unsafe { FastInput::from_slice(s.as_bytes()) };
    let t = unsafe { scanner.usize() };
    let mut sums = [0u64; 4];
    for start in (0..t).step_by(sums.len()) {
        let count = (t - start).min(sums.len());
        for sum in &mut sums[..count] {
            *sum = unsafe { scanner.u64() + scanner.u64() };
        }
        for &sum in &sums[..count] {
            writer.u64(sum);
            writer.byte(b'\n');
        }
    }
}
