pub mod lambda;
pub mod search;
pub mod types;
mod utils;

pub use lambda::*;
pub use search::*;
pub use types::*;
pub(crate) use utils::*;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
