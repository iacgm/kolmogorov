use super::*;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;

type Thunk = Rc<RefCell<NTerm>>;

#[derive(Clone, Debug)]
pub enum NTerm {
	Num(i32),
	Var(Identifier),
	Lam(Identifier, Rc<NTerm>),
	App(Thunk, Thunk),

	// Transparent indirection to another term.
	// Occasionally useful for a faithful implementation of graph reduction.
	// May eventually be removed during optimization.
	Ref(Thunk),
}

enum SpineCollapse {
	Whnf,
	Exec(BuiltIn, Vec<Thunk>),
}

impl Context {
	pub fn evaluate(&self, term: Term) -> NTerm {
		let term = NTerm::from(term);
		let mut thunk: Thunk = term.into();
		self.evaluate_thunk(&mut thunk);
		Rc::unwrap_or_clone(thunk).into_inner()
	}

	// True if any work done
	fn evaluate_thunk(&self, thunk: &mut Thunk) {
		use NTerm::*;
		let mut borrow = (**thunk).borrow_mut();
		let term = &mut *borrow;
		match term {
			Num(_) | Lam(_, _) => (),
			Var(v) => {
				if let Some(builtin) = self.get(v) {
					if builtin.n_args == 0 {
						let func = &*builtin.func;
						*term = func(&mut []).unwrap().into();
						drop(borrow);
						self.evaluate_thunk(thunk)
					}
				}
			}
			Ref(next) => {
				let next = next.clone();
				drop(borrow);
				*thunk = next;
				self.evaluate_thunk(thunk)
			}
			App(_, _) => {
				self.collapse_spine(term, 0);
			}
		}
	}

	fn collapse_spine(&self, root: &mut NTerm, depth: usize) -> SpineCollapse {
		use NTerm::*;
		use SpineCollapse::*;
		match root {
			Num(_) | Lam(_, _) => Whnf,
			Ref(thunk) => self.collapse_spine(&mut thunk.borrow_mut(), depth),
			Var(v) => {
				if let Some(builtin) = self.get(v) {
					if builtin.n_args == 0 {
						let func = &*builtin.func;
						*root = func(&mut []).unwrap().into();
						self.collapse_spine(root, depth)
					} else if depth >= builtin.n_args {
						Exec(builtin.clone(), vec![])
					} else {
						Whnf
					}
				} else {
					Whnf
				}
			}
			App(l, r) => {
				let borr = (**l).borrow();
				if let Lam(_, _) = *borr {
					let Lam(v, b) = borr.clone() else {
						unreachable!()
					};

					drop(borr);

					*root = instantiate(&b, v, r);

					self.collapse_spine(root, depth)
				} else {
					drop(borr);

					let mut spine = self.collapse_spine(&mut l.borrow_mut(), depth + 1);

					if let Exec(builtin, args) = &mut spine {
						let argc = builtin.n_args;

						args.push(r.clone());

						if argc == args.len() {
							for arg in &mut args[..] {
								self.evaluate_thunk(arg);
							}

							let mut terms: Vec<Term> = vec![];
							for arg in args.drain(..).rev() {
								let arg = Rc::unwrap_or_clone(arg).into_inner();
								terms.push(arg.into());
							}

							let func = &*builtin.func;

							if let Some(term) = func(&mut terms[..]) {
								*root = term.into();
							}
						}

						spine
					} else {
						self.collapse_spine(root, depth)
					}
				}
			}
		}
	}
}

fn instantiate(body: &NTerm, var: Identifier, thunk: &Thunk) -> NTerm {
	use NTerm::*;
	match &body {
		Num(n) => Num(*n),
		Lam(v, b) => {
			if *v == var {
				Lam(v, b.clone())
			} else {
				Lam(v, instantiate(b, var, thunk).into())
			}
		}
		Var(v) => {
			if *v == var {
				Ref(thunk.clone())
			} else {
				Var(v)
			}
		}
		Ref(next) => {
			let next = &*(**next).borrow();
			instantiate(next, var, thunk)
		}
		App(l, r) => App(
			instantiate(&l.borrow_mut(), var, thunk).into(),
			instantiate(&r.borrow_mut(), var, thunk).into(),
		),
	}
}

impl From<NTerm> for Thunk {
	fn from(value: NTerm) -> Self {
		Rc::new(value.into())
	}
}

impl From<Term> for NTerm {
	fn from(value: Term) -> Self {
		use NTerm::*;
		match value {
			Term::Num(n) => Num(n),
			Term::Var(v) => Var(v),
			Term::Lam(v, b) => Lam(v, Rc::new(Self::from(*b))),
			Term::App(mut apps) => {
				let mut res: NTerm = apps.pop().unwrap().into();
				while !apps.is_empty() {
					let arg: NTerm = apps.pop().unwrap().into();
					res = App(res.into(), arg.into());
				}
				res
			}
		}
	}
}

impl From<NTerm> for Term {
	fn from(value: NTerm) -> Self {
		use Term::*;
		match value {
			NTerm::Num(n) => Num(n),
			NTerm::Var(v) => Var(v),
			NTerm::Lam(v, b) => Lam(v, Box::new(Self::from((*b).borrow().clone()))),
			NTerm::Ref(next) => (*next).borrow().clone().into(),
			NTerm::App(l, r) => {
				let l = Self::from((*l).borrow().clone());
				let r = Self::from((*r).borrow().clone());
				l.applied_to(r)
			}
		}
	}
}

impl std::fmt::Display for NTerm {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Term::from(self.clone()).fmt(f)
	}
}
