pub mod entry;
mod ext;
mod format;
mod map;
mod parse;

pub use ext::{AttributeExt, ItemExt, PathExt};
pub use format::{format_with_rustfmt, rustfmt_exits};
pub use map::{Filter, LinkedSnippet, SnippetMap};
pub use parse::{parse_file_recursive, Error};
