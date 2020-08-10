#[macro_export]
macro_rules! echo {
    ($writer:expr, $iter:expr) => {
        echo!($writer, $iter, "\n")
    };
    ($writer:expr, $iter:expr, $sep:expr) => {
        let mut iter = $iter;
        if let Some(item) = iter.next() {
            write!($writer, "{}", item).ok();
        }
        for item in iter {
            write!($writer, "{}{}", $sep, item).ok();
        }
        writeln!($writer).ok();
    };
}
