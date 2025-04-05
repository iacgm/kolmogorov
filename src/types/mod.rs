pub mod subs;
pub use subs::*;

use super::*;

use rustc_hash::FxHashSet as HashSet;
use std::rc::Rc;

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Type {
	Var(Identifier),
	Fun(Rc<Type>, Rc<Type>),
}

impl Type {
	pub fn vars(&self) -> HashSet<Identifier> {
		use Type::*;
		match self {
			Var(v) => {
				let mut set = HashSet::default();
				set.insert(*v);
				set
			}
			Fun(l, r) => {
				let mut vars = l.vars();
				for v in r.vars() {
					vars.insert(v);
				}
				vars
			}
		}
	}
}

use std::fmt::*;
impl Display for Type {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		use Type::*;
		match self {
			Var(v) => write!(f, "{}", v),
			Fun(x, y) => {
				write!(f, "({}", x)?;
				let mut r = &**y;
				while let Fun(t, next) = r {
					write!(f, "=>{}", t)?;
					r = &**next;
				}
				write!(f, "=>{}", r)?;
				write!(f, ")")
			}
		}
	}
}

impl Debug for Type {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		Display::fmt(self, f)
	}
}

impl From<Identifier> for Type {
	fn from(ident: Identifier) -> Self {
		Self::Var(ident)
	}
}
