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

mod cache;
mod node;
use cache::*;
use node::*;

use super::*;

use std::rc::Rc;
use NodeKind::*;

//A series of applied terms, annotated with type
pub struct Searcher {
	ctx: Context,
	vgen: VarGen,
	calls: Vec<SearchNode>,
	arg_vars: Vec<(Identifier, Rc<Type>)>,
	cache: Cache,
}

impl Searcher {
	pub fn search(ctx: Context, targ: &Type, size: usize) -> Self {
		let vgen = ctx.vgen();

		Searcher {
			ctx,
			vgen,
			calls: vec![SearchNode {
				targ: targ.clone().into(),
				size,
				next: None,
				kind: START_KIND,
			}],
			arg_vars: vec![],
			cache: Cache::new(),
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
		use Phase::*;
		match kind {
			All(Body) => {
				if self.cache.prune(targ, *size) {
					self.calls.pop();
					return None;
				}

				if *size == 0 {
					self.calls.pop();
					return None;
				}

				*next = Some(len);
				*kind = All(Abstraction);

				let size = *size;
				let targ = targ.clone();

				let vars = self.vars_producing(&targ);

				let node = SearchNode {
					targ,
					size,
					next: None,
					kind: HeadVars(vars),
				};

				self.cache.begin_search(&node);

				self.calls.push(node);

				self.try_next_at(n + 1)
			}
			All(Abstraction) => {
				*next = Some(len);
				*kind = All(Completed);

				let node = SearchNode {
					targ: targ.clone(),
					size: *size,
					next: None,
					kind: Abstract,
				};
				self.calls.push(node);

				self.try_next_at(n + 1)
			}
			All(Completed) => {
				self.cache.end_search();
				self.calls.pop();
				None
			}

			Abstract if n == len - 1 => {
				let Type::Fun(arg, ret) = (**targ).clone() else {
					self.calls.pop();
					return None;
				};

				let size = *size;
				self.introduce_var(arg);

				let node = SearchNode {
					targ: ret.clone(),
					size: size - 1,
					next: None,
					kind: START_KIND,
				};

				self.calls.push(node);

				None
			}
			Abstract => {
				let size = *size;
				let ident = self.arg_vars.last().unwrap().0;

				let Some(body) = self.next_at(n + 1) else {
					self.eliminate_var();
					self.calls.pop();
					return None;
				};

				let output = Term::Lam(ident, Box::new(body));

				self.cache.yield_term(output, size)
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
				if self.cache.prune_arg(targ, l_ty, *size) {
					self.calls.pop();
					return None;
				}

				if *size == 0 && l_ty == targ {
					let term = apps.build_term();
					self.calls.pop();
					let size = term.size();
					return self.cache.yield_term(term, size);
				} else if *size == 0 || l_ty == targ {
					self.calls.pop();
					return None;
				}

				let Type::Fun(arg, ret) = &**l_ty else {
					self.calls.pop();
					return None;
				};

				let node = SearchNode {
					targ: arg.clone(),
					size: if ret == targ { *size - 1 } else { 1 },
					next: None,
					kind: START_KIND,
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
						None if arg_size == size - 1 => {
							self.calls.pop();
							return None;
						}
						None => {
							let node = SearchNode {
								targ: arg_ty.clone(),
								size: arg_size + 1,
								next: None,
								kind: START_KIND,
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

	fn vars_producing(&self, ty: &Type) -> VarsVec {
		let mut vec: VarsVec = self
			.ctx
			.iter()
			.filter_map(move |(&v, BuiltIn { ty: t, .. })| {
				if produces(t, ty) {
					Some((v, t.clone()))
				} else {
					None
				}
			})
			.collect();

		vec.extend(self.arg_vars.iter().filter_map(move |(v, t)| {
			if produces(t, ty) {
				Some((*v, t.clone()))
			} else {
				None
			}
		}));

		vec
	}

	fn introduce_var(&mut self, ty: Rc<Type>) {
		let ident = self.vgen.small_var();
		self.arg_vars.push((ident, ty.clone()));

		let is_new = !self.contains_var_of_ty(&ty);
		self.cache.intro_var(is_new);
	}

	fn eliminate_var(&mut self) {
		let (ident, _) = self.arg_vars.pop().unwrap();
		self.vgen.freshen(ident);
		self.cache.elim_var();
	}

	fn contains_var_of_ty(&self, ty: &Rc<Type>) -> bool {
		for (_, builtin) in self.ctx.iter() {
			if builtin.ty == *ty {
				return true;
			}
		}

		false
	}
}

impl Iterator for Searcher {
	type Item = Term;
	fn next(&mut self) -> Option<Term> {
		self.next_at(0)
	}
}

fn produces(ty: &Type, target: &Type) -> bool {
	let ret_ty_produces = match ty {
		Type::Fun(_, r) => produces(r, target),
		_ => false,
	};

	ret_ty_produces || target == ty
}
