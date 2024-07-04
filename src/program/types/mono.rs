use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MonoType {
	Int,
	Var(Identifier),
	Fun(Box<MonoType>, Box<MonoType>),
}

impl MonoType {
	pub fn unify(&self, other: &MonoType) -> Option<TypeSub> {
		use MonoType::*;

		let mut subs = HashMap::<Identifier, &MonoType>::default();
		let mut stack = vec![(self, other)];

		while let Some(pair) = stack.pop() {
			match pair {
				(Int, Int) => continue,
				(Var(x), Var(y)) if x == y => continue,
				(Var(x), Var(y)) if x != y => return None,
				(t, Var(v)) | (Var(v), t) => {
					if t.free_vars().contains(v) {
						return None
					}

					let Some(expected) = subs.get(v) else {
						subs.insert(v, t);
						continue;
					};

					stack.push((t, expected));
				}
				(Fun(lx, ly), Fun(rx, ry)) => {
					stack.push((lx, rx));
					stack.push((ly, ry));
				}
				_ => return None,
			}
		}

		let subs = TypeSub::from_iter(subs.into_iter().map(|(k, v)| (k, v.clone())));

		Some(subs)
	}

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

	pub fn poly(self) -> PolyType {
		PolyType {
			vars: self.free_vars(),
			mono: self,
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
