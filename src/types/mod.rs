pub mod subs;
pub use subs::*;

use super::*;

use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;
use std::rc::Rc;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Type {
	Int,
	Var(Identifier),
	Fun(Rc<Type>, Rc<Type>),
}

impl Type {
	pub fn instantiates(&self, other: &Self) -> bool {
		use Type::*;

		let mut bindings = HashMap::default();
		let mut stack = vec![(self, other)];

		while let Some(pair) = stack.pop() {
			match pair {
				(Int, Int) => continue,
				(Var(v), right) => {
					let Some(&expected) = bindings.get(v) else {
						bindings.insert(v, right);
						continue;
					};

					if expected != right {
						return false;
					}
				}
				(Fun(ld, lr), Fun(rd, rr)) => {
					stack.push((ld, rd));
					stack.push((lr, rr));
				}
				_ => return false,
			}
		}

		true
	}

	pub fn fresh(&self, vgen: &mut VarGen) -> Self {
		fn core(ty: &mut Type, map: &HashMap<Identifier, Identifier>) {
			use Type::*;
			match ty {
				Int => (),
				Var(v) => {
					if let Some(new) = map.get(v) {
						*ty = Var(new);
					}
				}
				Fun(l, r) => {
					core(Rc::make_mut(l), map);
					core(Rc::make_mut(r), map);
				}
			}
		}

		let mut map = HashMap::default();

		for v in self.vars() {
			map.insert(v, vgen.cap_var());
		}

		let mut clone = self.clone();
		core(&mut clone, &map);
		clone
	}

	pub fn vars(&self) -> HashSet<Identifier> {
		use Type::*;
		match self {
			Int => HashSet::default(),
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

	pub fn order(&self) -> Option<usize> {
		use Type::*;
		match self {
			Int => Some(0),
			Var(_) => None,
			Fun(l, r) => Some(r.order()?.max(l.order()? + 1)),
		}
	}
}

use std::fmt::*;
impl Display for Type {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		use Type::*;
		match self {
			Int => write!(f, "N"),
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
