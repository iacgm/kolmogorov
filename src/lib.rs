pub mod lambda;
pub mod search;
pub mod types;

pub use lambda::*;
pub use search::*;
pub use types::*;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
