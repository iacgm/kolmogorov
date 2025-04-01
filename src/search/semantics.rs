use std::{
	fmt::{Debug, Display},
	hash::Hash,
};

pub trait Semantics: Debug + Clone + Eq + Hash + Display {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OpaqueSemantics;

impl Semantics for OpaqueSemantics {}

impl Display for OpaqueSemantics {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}
