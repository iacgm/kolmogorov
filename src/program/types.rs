use std::collections::HashSet;

#[derive(Clone, Debug)]
pub enum Type {
	Nat,
	Var(&'static str),
	Fun(Box<Type>, Box<Type>),
}

impl Type {
	pub fn free_vars(&self) -> HashSet<&'static str> {
		use Type::*;
		match &self {
			Nat => HashSet::new(),
			Var(c) => HashSet::from([*c]),
			Fun(l, r) => {
				let l = l.free_vars();
				let r = r.free_vars();
				l.union(&r).copied().collect::<_>()
			}
		}
	}
}
