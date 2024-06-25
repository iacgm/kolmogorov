use std::collections::HashSet;

#[derive(Clone, Debug)]
pub enum Type {
	Nat,
	Var(&'static str),
	Fun(Box<Type>, Box<Type>),
	//Polymorphic type (i.e., forall a: t)
	All(&'static str, Box<Type>)
}

impl Type {
	pub fn free_vars(&self) -> HashSet<&'static str> {
		use Type::*;
		match &self {
			Nat => HashSet::new(),
			Var(v) => HashSet::from([*v]),
			All(v, t) => {
				let mut free = t.free_vars();
				free.remove(v);
				free
			},
			Fun(l, r) => {
				let l = l.free_vars();
				let r = r.free_vars();
				l.union(&r).copied().collect::<_>()
			}
		}
	}
}
