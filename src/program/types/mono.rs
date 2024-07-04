use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MonoType {
	Int,
	Var(Identifier),
	Fun(Box<MonoType>, Box<MonoType>),
}

impl MonoType {
	pub fn sub(&mut self, old: Identifier, new: &MonoType) {
		use MonoType::*;
		match self {
			Var(i) if *i == old => *self = new.clone(),
			Fun(x, y) => {
				x.sub(old, new);
				y.sub(old, new);
			}
			_ => (),
		}
	}

	pub fn free_vars(&self) -> HashSet<Identifier> {
		use MonoType::*;
		match &self {
			Int => HashSet::new(),
			Var(v) => HashSet::from([*v]),
			Fun(l, r) => {
				let l = l.free_vars();
				let r = r.free_vars();
				l.union(&r).copied().collect()
			}
		}
	}
}

impl std::fmt::Display for MonoType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use MonoType::*;
		match self {
			Int => write!(f, "N"),
			Var(v) => write!(f, "{}", v),
			Fun(x, y) => write!(f, "({}=>{})", x, y),
		}
	}
}
