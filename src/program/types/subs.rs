use super::*;
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct TypeSub {
	dict: HashMap<Identifier, MonoType>,
}

impl TypeSub {
	pub fn unify(&mut self, lhs: &MonoType, rhs: &MonoType) -> bool {
		use MonoType::*;

		type RefSub<'a> = HashMap<Identifier, &'a MonoType>;

		fn contains(sub: &RefSub, s: Identifier, t: &MonoType) -> bool {
			match t {
				Int => false,
				Var(v) if *v == s => true,
				Var(v) => {
					if let Some(t) = sub.get(v) {
						contains(sub, s, t)
					} else {
						false
					}
				}
				Fun(l, r) => contains(sub, s, l) || contains(sub, s, r),
			}
		}

		let mut subs = RefSub::default();
		let mut stack = vec![(lhs, rhs)];

		while let Some(pair) = stack.pop() {
			match pair {
				(Int, Int) => continue,
				(Var(x), Var(y)) if x == y => continue,
				(t, Var(v)) | (Var(v), t) => {
					if contains(&subs, v, t) {
						return false;
					}

					if let Some(expected) = self.dict.get(v) {
						stack.push((t, expected));
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
				_ => return false,
			}
		}

		let news: Vec<_> = subs
			.into_iter()
			.map(|(id, mono)| (id, mono.clone()))
			.collect();

		for (id, mono) in news {
			self.dict.insert(id, mono.clone());
		}

		true
	}

	pub fn to_mono(&self, mono: &mut MonoType) {
		use MonoType::*;
		match mono {
			Int => (),
			Var(v) => {
				if let Some(ty) = self.dict.get(v) {
					*mono = ty.clone();
					self.to_mono(mono);
				}
			}
			Fun(l, r) => {
				self.to_mono(l);
				self.to_mono(r);
			}
		}
	}
}
