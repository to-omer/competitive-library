use super::{Allocator, MemoryPool};
#[codesnip::entry("SplayMap")]
pub use sized_map::SplayMap;

#[cfg_attr(nightly, codesnip::entry("splay_node", include("Allocator")))]
pub mod node;
#[cfg_attr(nightly, codesnip::entry("SplayMap", include("splay_node")))]
pub mod sized_map;
