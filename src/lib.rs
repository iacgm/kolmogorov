pub mod lambda;
pub mod search;
mod utils;

pub use lambda::*;
pub use search::*;
pub(crate) use utils::*;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
