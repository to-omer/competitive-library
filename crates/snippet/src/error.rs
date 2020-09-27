#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("parse error: {0}")]
    CompileError(#[from] syn::Error),
    #[error("module `{0}` not found")]
    ModuleNotFound(String),
}

pub type ParseResult<T> = Result<T, ParseError>;
