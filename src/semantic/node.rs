use std::fmt::Debug;

//Abstracted because we will try later to eliminate as many unnecessary allocs as possible
use super::*;
use std::rc::Rc;

#[derive(Debug)]
pub struct SearchNode {
	pub targ: Rc<Type>,      //Target type
	pub size: usize,         //Size
	pub next: Option<usize>, //Short circuit to next node, if it exists
	pub kind: NodeKind,
}

#[derive(Clone)]
pub enum NodeKind {
	All(bool), //bool to indicate whether this node has been visited
	ArgTo(Stack<Term>, Rc<Type>),
	HeadVars(Vec<(Identifier, Rc<Type>)>),
}

impl Debug for NodeKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use NodeKind::*;
		match self {
			All(b) => write!(f, "All({})", b),
			ArgTo(s, t) => write!(f, "ArgTo({:?}, {})", s, t),
			HeadVars(vs) => write!(f, "Vars({:?})", vs),
		}
	}
}
