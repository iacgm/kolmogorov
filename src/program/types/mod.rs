pub mod context;
pub mod mono;

use std::collections::HashMap;

pub use context::*;
pub use mono::*;

use super::*;

#[derive(Clone, Debug)]
pub struct PolyType {
	pub vars: HashSet<Identifier>,
	pub mono: MonoType,
}

impl PolyType {
	pub fn matches(&self, mono: &MonoType) -> bool {
		use MonoType::*;

		let free = &self.vars;

		let mut bindings = HashMap::<&str, &MonoType>::new();
		let mut checks = vec![(&self.mono, mono)];

		while let Some(pair) = checks.pop() {
			match pair {
				(Var(v), right) if free.contains(v) => {
					let Some(&expected) = bindings.get(v) else {
						bindings.insert(v, right);
						continue;
					};
					
					if expected != right {
						return false;
					}
				},
				(Var(a), Var(b)) if a != b => return false,
				(Fun(lx, ly), Fun(rx, ry)) => {
					checks.push((lx, rx));
					checks.push((ly, ry));
				}
				_ => continue,
			}
		}

		true
	}

	pub fn func(mut from: PolyType, to: PolyType) -> Self {
		from.distinguish_from(&to);

		let mono = MonoType::Fun(from.mono.into(), to.mono.into());

		Self {
			vars: from.vars.union(&to.vars).copied().collect(),
			mono,
		}
	}

	fn distinguish_from(&mut self, other: &Self) {
		use MonoType::*;

		let intersection: Vec<_> = self.vars.intersection(&other.vars).copied().collect();
		for old in intersection {
			let is_fresh = |v| !self.vars.contains(v) && !other.vars.contains(v);
			let new = new_var_where(is_fresh).unwrap();
			self.mono.sub(old, &Var(new));
			self.vars.remove(old);
			self.vars.insert(new);
		}
	}
}

impl std::fmt::Display for PolyType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if !self.vars.is_empty() {
			write!(f, "forall")?;
			for var in self.vars.iter() {
				write!(f, " {}", var)?;
			}
			write!(f, " :: ")?;
		}

		write!(f, "{}", self.mono)
	}
}

impl From<MonoType> for PolyType {
	fn from(mono: MonoType) -> Self {
		Self {
			vars: mono.free_vars(),
			mono,
		}
	}
}
