#[cargo_snippet::snippet]
pub fn echo<T: std::fmt::Display>(
    writer: &mut impl std::io::Write,
    iter: impl IntoIterator<Item = T>,
    sep: impl std::fmt::Display,
) -> std::io::Result<()> {
    let mut iter = iter.into_iter();
    if let Some(item) = iter.next() {
        write!(writer, "{}", item)?;
    }
    for item in iter {
        write!(writer, "{}{}", sep, item)?;
    }
    writeln!(writer)
}
