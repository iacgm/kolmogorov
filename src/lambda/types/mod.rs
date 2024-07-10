pub mod subs;

pub use subs::*;

use super::*;

#[derive(Debug, Clone)]
pub enum Type {
	Int,
	Var(Identifier),
	Fun(Box<Type>, Box<Type>),
}

impl Type {
	pub fn freshen(&mut self, var_gen: &mut VarGen) {
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
			map.insert(v, var_gen.newvar());
		}

		core(self, &map);
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
