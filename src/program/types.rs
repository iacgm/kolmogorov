use std::collections::HashSet;

#[derive(Clone, Debug)]
pub enum Type {
	Int,
	Name(&'static str),
	Func(Box<Type>, Box<Type>),
	//Polymorphic type (i.e., forall a: t)
	Poly(&'static str, Box<Type>)
}

impl Type {
	pub fn free_vars(&self) -> HashSet<&'static str> {
		use Type::*;
		match &self {
			Int => HashSet::new(),
			Name(v) => HashSet::from([*v]),
			Poly(v, t) => {
				let mut free = t.free_vars();
				free.remove(v);
				free
			},
			Func(l, r) => {
				let l = l.free_vars();
				let r = r.free_vars();
				l.union(&r).copied().collect::<_>()
			}
		}
	}
}
