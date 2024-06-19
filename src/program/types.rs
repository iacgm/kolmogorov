use std::collections::HashSet;

pub enum Type {
	Int,
	Var(char),
	Fun(Box<Type>, Box<Type>),
}

impl Type {
	pub fn free_vars(&self) -> HashSet<char> {
		use Type::*;
		match &self {
			Int => HashSet::new(),
			Var(c) => HashSet::from([*c]),
			Fun(l, r) => {
				let l = l.free_vars();
				let r = r.free_vars();
				l.union(&r).copied().collect::<_>()
			}
		}
	}
}

#[macro_export]
macro_rules! ty {
	(N) => {
		Type::Int
	};
	($x:literal) => {
		Type::Var($x)
	};
	(($a:tt) -> ($b:tt)) => {
		Type::Fun(ty!($a), ty!($b))
	};
	(($r:tt)) => {
		ty!($r)
	};
}
