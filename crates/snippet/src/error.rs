use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(#[from] syn::Error),
    #[error("Failed to parse ")]
    ParseFile(PathBuf, #[source] syn::Error),
    #[error("Module `{0}` not found where `{}`.", .1.display())]
    ModuleNotFound(String, PathBuf),
    #[error("File `{}` not found.", .0.display())]
    FileNotFound(PathBuf, #[source] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
