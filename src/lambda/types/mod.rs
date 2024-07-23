pub mod subs;

pub use subs::*;

use super::*;

#[derive(Clone, PartialEq)]
pub enum Type {
	Int,
	Var(Identifier),
	Fun(Box<Type>, Box<Type>),
}

impl Type {
	pub fn instantiates(&self, other: &Self) -> bool {
		use Type::*;

		let mut bindings = HashMap::new();
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
					core(l, map);
					core(r, map);
				}
			}
		}

		let mut map = HashMap::new();

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
			Int => HashSet::new(),
			Var(v) => HashSet::from([*v]),
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

use std::{collections::HashMap, fmt::*};
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
