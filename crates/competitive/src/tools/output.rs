#[codesnip::entry("_echo")]
pub fn echo<T: std::fmt::Display>(
    mut writer: impl std::io::Write,
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

/// Struct for formatting an Iterator with separator.
#[derive(Debug, Clone)]
pub struct Echo<Iter, Sep>(pub Iter, pub Sep);
impl<Iter> Echo<Iter, char> {
    pub fn line(iter: Iter) -> Self {
        Self(iter, '\n')
    }
    pub fn white(iter: Iter) -> Self {
        Self(iter, ' ')
    }
}
impl<Iter> Echo<Iter, &'static str> {
    pub fn none(iter: Iter) -> Self {
        Self(iter, "")
    }
}
impl<Iter, E, Sep> std::fmt::Display for Echo<Iter, Sep>
where
    Iter: Clone + IntoIterator<Item = E>,
    E: std::fmt::Display,
    Sep: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.0.clone().into_iter();
        if let Some(item) = iter.next() {
            write!(f, "{}", item)?;
        }
        for item in iter {
            write!(f, "{}{}", self.1, item)?;
        }
        writeln!(f)
    }
}
