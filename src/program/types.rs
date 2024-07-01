use super::*;

#[derive(Clone, Debug)]
pub struct PolyType {
	pub vars: HashSet<Ident>,
	pub mono: MonoType,
}

#[derive(Clone, Debug)]
pub enum MonoType {
	Int,
	Name(Ident),
	Func(Box<MonoType>, Box<MonoType>),
}

impl PolyType {
	pub fn func(mut from: PolyType, to: PolyType) -> Self {
		from.distinguish_from(&to);

		let mono = MonoType::Func(from.mono.into(), to.mono.into());

		Self {
			vars: from.vars.union(&to.vars).copied().collect(),
			mono,
		}
	}

	fn distinguish_from(&mut self, other: &Self) {
		let intersection: Vec<_> = self.vars.intersection(&other.vars).copied().collect();
		for old in intersection {
			let is_fresh = |v| !self.vars.contains(v) && !other.vars.contains(v);
			let new = new_var_where(is_fresh).unwrap();
			self.mono.rename(old, new);
			self.vars.remove(old);
			self.vars.insert(new);
		}
	}
}

impl MonoType {
	pub fn rename(&mut self, old: Ident, new: Ident) {
		use MonoType::*;
		match self {
			Name(i) if *i == old => *self = Name(new),
			Func(x, y) => {
				x.rename(old, new);
				y.rename(old, new);
			}
			_ => (),
		}
	}

	pub fn free_vars(&self) -> HashSet<Ident> {
		use MonoType::*;
		match &self {
			Int => HashSet::new(),
			Name(v) => HashSet::from([*v]),
			Func(l, r) => {
				let l = l.free_vars();
				let r = r.free_vars();
				l.union(&r).copied().collect()
			}
		}
	}
}

impl From<MonoType> for PolyType {
	fn from(mono: MonoType) -> Self {
		Self {
			vars: Default::default(),
			mono,
		}
	}
}
