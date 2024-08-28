use super::*;
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
			Num(_) | Lam(_, _) | Var(_) => (),
			Ref(next) => {
				let next = next.clone();
				drop(borrow);
				*thunk = next;
				self.evaluate_thunk(thunk)
			}
			App(l, r) => {
				let borr = l.borrow();
				if let Lam(_, _) = *borr {
					let Lam(v, mut b) = borr.clone() else {
						unreachable!()
					};

					drop(borr);

					Rc::make_mut(&mut b).sub(v, r);
					*term = (*b).clone();

					drop(borrow);

					self.evaluate_thunk(thunk);
				} else {
					drop(borr);

					self.evaluate_thunk(l);
					self.evaluate_thunk(r);

					drop(borrow);
					self.evaluate_thunk(thunk);
				}
			}
		}
	}
}

impl NTerm {
	pub fn sub(&mut self, var: Identifier, thunk: &Thunk) {
		use NTerm::*;
		match self {
			Lam(v, b) if *v != var => {
				Rc::make_mut(b).sub(var, thunk);
			}
			App(l, r) => {
				(**l).borrow_mut().sub(var, thunk);
				(**r).borrow_mut().sub(var, thunk);
			}
			Var(v) if *v == var => *self = Ref(thunk.clone()),
			_ => (),
		}
	}
}

impl From<NTerm> for Thunk {
	fn from(value: NTerm) -> Self {
		Rc::new(value.into())
	}
}

impl From<Term> for NTerm {
	fn from(value: Term) -> Self {
		use Term::*;
		match value {
			Num(n) => Self::Num(n),
			Var(v) => Self::Var(v),
			Lam(v, b) => Self::Lam(v, Rc::new(Self::from(*b))),
			App(mut apps) => {
				let mut res: NTerm = apps.pop().unwrap().into();
				while !apps.is_empty() {
					let arg: NTerm = apps.pop().unwrap().into();
					res = Self::App(res.into(), arg.into());
				}
				res
			}
		}
	}
}
