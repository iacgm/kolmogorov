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

use super::*;

mod node;
use node::*;

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
				targ: targ.clone(),
				size,
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

		let SearchNode { targ, size, kind } = &mut self.calls[n];

		use NodeKind::*;
		match kind {
			All(false) if n == len - 1 => {
				let vars = vars_producing(&self.dict, targ);

				*kind = All(true);
				let node = SearchNode {
					targ: targ.clone(),
					size: *size,
					kind: HeadVars(vars),
				};
				self.calls.push(node);

				self.next_at(n + 1)
			}
			All(false) => self.next_at(n + 1),
			All(true) if n == len - 1 => {
				self.calls.pop();
				None
			}
			All(true) => self.next_at(n + 1),

			HeadVars(_) if *size == 0 => {
				self.calls.pop();
				None
			}
			HeadVars(vars) if n == len - 1 => {
				let Some((var, v_ty)) = vars.pop() else {
					self.calls.pop();
					return None;
				};

				let node = SearchNode {
					targ: targ.clone(),
					size: *size - 1,
					kind: ArgTo(Stack::one(Term::Var(var)), v_ty, None),
				};

				self.calls.push(node);
				None
			}
			HeadVars(_) => self.next_at(n + 1),

			ArgTo(apps, l_ty, None) if n == len - 1 => {
				let done = l_ty == targ;

				if *size == 0 && done {
					let term = Term::App(apps.rev_vec());
					self.calls.pop();
					return Some(term);
				} else if *size == 0 || done {
					self.calls.pop();
					return None;
				}

				let Type::Fun(arg, _) = l_ty else {
					self.calls.pop();
					return None;
				};

				let node = SearchNode {
					targ: *arg.clone(),
					size: *size - 1,
					kind: NodeKind::All(false),
				};

				self.calls.push(node);

				None
			}
			ArgTo(apps, l_ty, None) => {
				let Type::Fun(arg_ty, ret) = l_ty else {
					unreachable!()
				};

				let arg_ty = *arg_ty.clone();
				let targ = targ.clone();
				let size = *size;
				let ret = *ret.clone();
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
								kind: All(false),
							};
							self.calls.push(node);
						}
					}
				};

				let len = self.calls.len();
				let ArgTo(_, _, next) = &mut self.calls[n].kind else {
					unreachable!()
				};

				*next = Some(len);

				let node = SearchNode {
					targ,
					size: size - arg_size - 1,
					kind: ArgTo(apps.cons(arg), ret, None),
				};

				self.calls.push(node);
				None
			}
			ArgTo(_, _, next) => {
				let p = next.unwrap();
				if p < len {
					self.next_at(p)
				} else {
					*next = None;
					None
				}
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

fn vars_producing(dict: &Dictionary, ty: &Type) -> Vec<(Identifier, Type)> {
	dict.iter_defs()
		.filter_map(move |(&v, def)| match def {
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
