/*
Enumerate programs, using some basic semantic information to narrow search space.
We try to have each extensionally-equal program appear duplicated as few times as
possible, ideally retaining only its shortest representation (regardless of its
runtime).

Rules that could be used to prune search tree are:
	> Reject paths which are known to be unsatisifiable early,
	  by counting programs of each size & type.
	> Reject programs which contain overly verbose representations
	  of constants
	> Reject programs that contain inverse operations (or other
	  unnecessary code) such as pred(succ(x))
	> Unnecessary repetition (folds on constant functions, for example)
*/

mod node;
use node::*;

use super::*;

use NodeKind::*;

//A series of applied terms, annotated with type
pub struct Searcher {
	dict: Dictionary,
	calls: Vec<SearchNode>,
}

impl Searcher {
	pub fn search(dict: &Dictionary, targ: &Type, size: usize) -> Self {
		Searcher {
			dict: dict.clone(),
			calls: vec![SearchNode {
				targ: targ.clone().into(),
				size,
				next: None,
				kind: All(false),
			}],
		}
	}

	fn next_at(&mut self, n: usize) -> Option<Term> {
		loop {
			if self.calls.len() <= n {
				break None;
			}

			if let Some(out) = self.try_next_at(n) {
				break Some(out);
			}
		}
	}

	fn try_next_at(&mut self, n: usize) -> Option<Term> {
		let len = self.calls.len();

		let SearchNode {
			targ,
			size,
			next,
			kind,
		} = &mut self.calls[n];

		if let Some(p) = next {
			let p = *p;
			if p < len {
				return self.try_next_at(p);
			} else {
				*next = None;
			}
		}

		use NodeKind::*;
		match kind {
			All(false) => {
				*next = Some(len);

				let vars = vars_producing(&self.dict, targ);

				*kind = All(true);
				let node = SearchNode {
					targ: targ.clone(),
					size: *size,
					next: None,
					kind: HeadVars(vars),
				};
				self.calls.push(node);

				self.next_at(n + 1)
			}
			All(true) => {
				self.calls.pop();
				None
			}

			HeadVars(_) if *size == 0 => {
				self.calls.pop();
				None
			}
			HeadVars(vars) => {
				*next = Some(n + 1);

				let Some((var, v_ty)) = vars.pop() else {
					self.calls.pop();
					return None;
				};

				let node = SearchNode {
					targ: targ.clone(),
					size: *size - 1,
					next: None,
					kind: ArgTo(Stack::one(Term::Var(var)), v_ty),
				};

				self.calls.push(node);
				None
			}

			ArgTo(apps, l_ty) if n == len - 1 => {
				if *size == 0 && l_ty == targ {
					let term = apps.build_term();
					self.calls.pop();
					return Some(term);
				} else if *size == 0 || l_ty == targ {
					self.calls.pop();
					return None;
				}

				let Type::Fun(arg, _) = &**l_ty else {
					self.calls.pop();
					return None;
				};

				let node = SearchNode {
					targ: arg.clone(),
					size: *size - 1,
					next: None,
					kind: NodeKind::All(false),
				};

				self.calls.push(node);

				None
			}
			ArgTo(apps, l_ty) => {
				let Type::Fun(arg_ty, ret) = &**l_ty else {
					unreachable!()
				};

				let arg_ty = arg_ty.clone();
				let targ = targ.clone();
				let size = *size;
				let ret = ret.clone();
				let apps = apps.clone();

				let (arg, arg_size) = loop {
					let arg_size = self.calls[n + 1].size;
					match self.next_at(n + 1) {
						Some(arg) => break (arg, arg_size),
						None if arg_size == 0 || ret == targ => {
							self.calls.pop();
							return None;
						}
						None => {
							let node = SearchNode {
								targ: arg_ty.clone(),
								size: arg_size - 1,
								next: None,
								kind: All(false),
							};
							self.calls.push(node);
						}
					}
				};

				let len = self.calls.len();

				self.calls[n].next = Some(len);

				let node = SearchNode {
					targ,
					size: size - arg_size - 1,
					next: None,
					kind: ArgTo(apps.cons(arg), ret),
				};

				self.calls.push(node);
				None
			}
		}
	}
}

impl Iterator for Searcher {
	type Item = Term;
	fn next(&mut self) -> Option<Term> {
		self.next_at(0)
	}
}

fn vars_producing(dict: &Dictionary, ty: &Type) -> VarsVec {
	dict.iter_defs()
		.filter_map(move |(v, def)| match def {
			Def::BuiltIn(_, t) if produces(t, ty) => Some((v, t.clone())),
			_ => None,
		})
		.collect()
}

fn produces(ty: &Type, target: &Type) -> bool {
	target == ty
		|| match ty {
			Type::Fun(_, r) => produces(r, target),
			_ => false,
		}
}
