use std::fmt::Debug;

//Abstracted because we will try later to eliminate as many unnecessary allocs as possible
use super::*;

#[derive(Debug)]
pub struct SearchNode {
	pub targ: Type,  //Target type
	pub size: usize, //Size 
	pub kind: NodeKind,
}

#[derive(Clone)]
pub enum NodeKind {
	All(bool), //bool to indicate whether this node has been visited
	ArgTo(Stack<Term>, Type, Option<usize>),
	HeadVars(Vec<(Identifier, Type)>),
}

impl Debug for NodeKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use NodeKind::*;
		match self {
			All(b) => write!(f, "All({})", b),
			ArgTo(s, t, o) => write!(f, "ArgTo({:?}, {}, {:?})", s, t, o),
			HeadVars(vs) => write!(f, "Vars({:?})", vs),
		}
	}
}
