use std::{
	fmt::{Debug, Display},
	hash::Hash,
};

pub trait Semantics: Debug + Clone + Eq + Hash + Display {}
