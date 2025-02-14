use std::{
	fmt::{Debug, Display},
	hash::Hash,
};

use super::*;

pub trait Semantics: Debug + Clone + Eq + Hash + Display {}
