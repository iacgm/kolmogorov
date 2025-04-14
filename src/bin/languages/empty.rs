use super::*;

use kolmogorov::*;

#[derive(Clone, Copy, Debug)]
pub struct Empty;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EmptySems;

impl Language for Empty {
    type Semantics = EmptySems;
    fn context(&self) -> Context {
        context! {}
    }
}

impl std::fmt::Display for EmptySems {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "()")
    }
}
