use competitive::prelude::*;

#[verify::library_checker("many_aplusb_128bit")]
pub fn many_aplusb_128bit(reader: impl Read, writer: impl Write) {
    prepare_io!(buffered; reader, writer);
    sc!(t: usize);
    let mut sums = [0i128; 4];
    for start in (0..t).step_by(sums.len()) {
        let count = (t - start).min(sums.len());
        for sum in &mut sums[..count] {
            sc!(a: i128, b: i128);
            *sum = a + b;
        }
        for &sum in &sums[..count] {
            pp!(sum);
        }
    }
}
