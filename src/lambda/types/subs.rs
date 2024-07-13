use super::*;
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct TypeSub {
	dict: HashMap<Identifier, Type>,
}

impl TypeSub {	
	pub fn unify(&mut self, lhs: &Type, rhs: &Type) -> bool {
		use Type::*;

		type RefSub<'a> = HashMap<Identifier, &'a Type>;

		fn contains(sub: &RefSub, s: Identifier, t: &Type) -> bool {
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

	pub fn apply(&self, ty: &mut Type) {
		use Type::*;
		match ty {
			Int => (),
			Var(v) => {
				if let Some(new) = self.dict.get(v) {
					*ty = new.clone();
					self.apply(ty);
				}
			}
			Fun(l, r) => {
				self.apply(l);
				self.apply(r);
			}
		}
	}
}
