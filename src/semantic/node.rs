use super::*;

use smallvec::SmallVec;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct SearchNode {
	pub targ: Rc<Type>,      //Target type
	pub size: usize,         //Size
	pub next: Option<usize>, //Short circuit to next node, if it exists
	pub kind: NodeKind,
}

type VarDef = (Identifier, Rc<Type>);
pub type VarsVec = SmallVec<[VarDef; 4]>;

//Top level search phases
#[derive(Clone, Debug)]
pub enum Phase {
	Body,
	Abstraction,
	Completed,
}

#[derive(Clone)]
pub enum NodeKind {
	All(Phase),
	ArgTo(Stack<Term>, Rc<Type>),
	HeadVars(VarsVec),
	Abstract,
}

pub const START_KIND : NodeKind = All(Phase::Body);

impl Debug for NodeKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use NodeKind::*;
		match self {
			All(b) => write!(f, "All({:?})", b),
			ArgTo(s, t) => write!(f, "ArgTo({:?}, {})", s, t),
			HeadVars(vs) => write!(f, "Vars({:?})", vs),
			Abstract => write!(f, "Abstract"),
		}
	}
}
