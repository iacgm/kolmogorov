use super::*;
use std::cell::RefCell;
use std::rc::Rc;

pub type Thunk = Rc<RefCell<Term>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Term {
	Num(i32),
	Var(Identifier),
	Lam(Identifier, Rc<Term>),
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
	pub fn evaluate(&self, term: &Term) -> Term {
		let mut thunk: Thunk = term.clone().into();
		self.evaluate_thunk(&mut thunk);
		Rc::unwrap_or_clone(thunk).into_inner()
	}

	// True if any work done
	pub fn evaluate_thunk(&self, thunk: &mut Thunk) {
		use Term::*;
		let mut borrow = (**thunk).borrow_mut();
		let term = &mut *borrow;
		match term {
			Num(_) | Lam(_, _) => (),
			Var(v) => {
				if let Some(BuiltIn {
					func, n_args: 0, ..
				}) = self.get(v)
				{
					*term = func(&mut []).unwrap();
					drop(borrow);
					self.evaluate_thunk(thunk)
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

	fn collapse_spine(&self, root: &mut Term, depth: usize) -> SpineCollapse {
		use Term::*;
		use SpineCollapse::*;
		match root {
			Ref(thunk) => self.collapse_spine(&mut thunk.borrow_mut(), depth),
			Num(_) | Lam(_, _) => Whnf,
			Var(v) => match self.get(v) {
				Some(BuiltIn {
					func, n_args: 0, ..
				}) => {
					*root = func(&mut []).unwrap();
					self.collapse_spine(root, depth)
				}
				Some(blt) if blt.n_args <= depth => {
					Exec(blt.clone(), Vec::with_capacity(blt.n_args))
				}
				_ => Whnf,
			},
			App(l, r) => {
				let mut borr = l.borrow_mut();
				match self.collapse_spine(&mut borr, depth + 1) {
					Exec(builtin, mut args) => {
						drop(borr);
						let argc = builtin.n_args;

						args.push(r.clone());

						if argc == args.len() {
							for arg in &mut args[..] {
								self.evaluate_thunk(arg);
							}

							let func = &*builtin.func;

							if let Some(term) = func(&mut args[..]) {
								*root = term;
								return self.collapse_spine(root, depth);
							}
						}

						Exec(builtin, args)
					}
					Whnf => {
						drop(borr);
						let borr = (**l).borrow();
						if let Lam(_, _) = *borr {
							let Lam(v, b) = borr.clone() else {
								unreachable!()
							};

							drop(borr);

							*root = b.instantiate(v, r);
							self.collapse_spine(root, depth)
						} else {
							drop(borr);
							Whnf
						}
					}
				}
			}
		}
	}
}

impl Term {
	fn instantiate(&self, var: Identifier, thunk: &Thunk) -> Term {
		use Term::*;
		match self {
			Num(n) => Num(*n),
			Lam(v, b) => {
				if *v == var {
					Lam(v, b.clone())
				} else {
					Lam(v, b.instantiate(var, thunk).into())
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
				next.instantiate(var, thunk)
			}
			App(l, r) => App(
				(**l).borrow().instantiate(var, thunk).into(),
				(**r).borrow().instantiate(var, thunk).into(),
			),
		}
	}

	pub fn size(&self) -> usize {
		use Term::*;
		match self {
			Ref(r) => (**r).borrow().size(),
			Num(_) | Var(_) => 1,
			Lam(_, b) => 1 + b.size(),
			App(l, r) => 1 + l.borrow().size() + r.borrow().size(),
		}
	}

	pub fn int(&self) -> Option<i32> {
		use Term::*;
		match self {
			Ref(r) => (**r).borrow().int(),
			Num(n) => Some(*n),
			_ => None,
		}
	}
}

impl From<Term> for Thunk {
	fn from(value: Term) -> Self {
		Rc::new(value.into())
	}
}

impl std::fmt::Display for Term {
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use Term::*;
		match self {
			Ref(r) => write!(fmt, "{}", (**r).borrow()),
			Num(k) => write!(fmt, "{}", k),
			Var(v) => write!(fmt, "{}", v),
			Lam(v, b) => {
				write!(fmt, "(\\{}", v)?;
				let mut r = &**b;
				while let Lam(v, next) = r {
					write!(fmt, " {}", v)?;
					r = &**next;
				}
				write!(fmt, " -> {}", r)?;
				write!(fmt, ")")
			}
			App(l, r) => {
				write!(fmt, "{}({})", (**l).borrow(), (**r).borrow())
			}
		}
	}
}
