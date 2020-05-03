#[allow(unused_imports)]
#[cargo_snippet::snippet("with_bufwriter")]
use std::io::Write;
#[cargo_snippet::snippet("with_bufwriter")]
pub fn with_bufwriter<F>(f: F) -> std::io::Result<()>
where
    F: FnOnce(&mut std::io::BufWriter<std::io::StdoutLock>) -> std::io::Result<()>,
{
    let stdout = std::io::stdout();
    let mut writer = std::io::BufWriter::new(stdout.lock());
    f(&mut writer)
}

// #[test]
// fn with_bufwriter_test() {
//     with_bufwriter(|out| {
//         for i in 0..10 {
//             write!(out, "{}", i)?;
//         }
//         Ok(())
//     })
//     .unwrap();
// }
