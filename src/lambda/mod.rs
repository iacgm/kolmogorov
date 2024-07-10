pub mod dictionary;
pub mod parser;
pub mod types;
mod vars;

pub use dictionary::*;
pub use parser::*;
pub use types::*;

use vars::*;
use std::collections::HashSet;

#[derive(Clone)]
pub enum Term {
	Num(i32),
	Var(Identifier),
	Lam(Identifier, Box<Term>),
	//Backwards representation of applications to facilitate
	//pushing & popping from the front
	App(Vec<Term>),
}

impl Term {
	pub fn exec(&mut self, context: &mut Dictionary) -> &mut Self {
		use Term::*;
		loop {
			match self {
				Num(_) | Var(_) => break,
				Lam(v, b) => {
					//Ensure v is treated as free
					context.shadow(v);
					b.exec(context);
					context.unshadow(v);
					break;
				}
				App(terms) => {
					match &terms[..] {
						[.., App(_)] => {
							let Some(App(start)) = terms.pop() else {
								unreachable!()
							};

							terms.extend(start);
							continue;
						}
						[_] => {
							let Some(inner) = terms.pop() else {
								unreachable!()
							};
							*self = inner;
							continue;
						}
						_ => (),
					}

					if context.reduce(terms) {
						continue;
					}

					if !self.head_red() {
						continue;
					}

					if !self.beta() {
						continue;
					}

					let App(terms) = self else { unreachable!() };

					for term in terms.iter_mut().rev() {
						term.exec(context);
					}

					break;
				}
			}
		}

		self
	}

	pub fn solve(&mut self, context: &mut Dictionary) -> &mut Self {
		use Term::*;
		loop {
			self.hnf();
			match self {
				App(terms) => {
					if !context.reduce(terms) {
						break;
					}
				}
				_ => break,
			}
		}
		self
	}

	pub fn sub(&mut self, var: Identifier, code: Term) {
		use Term::*;
		match self {
			Var(x) if *x == var => *self = code,
			Lam(x, b) if *x != var => {
				let free = code.free_vars();
				if free.contains(x) {
					let new = new_var_where(|s| !free.contains(s)).unwrap();
					b.sub(x, Var(new));
					*x = new;
				}
				b.sub(var, code);
			}
			App(t) => {
				for e in t {
					let code = code.clone();
					e.sub(var, code);
				}
			}
			_ => (),
		}
	}

	//A singular head reduction, returns true if in head normal form
	pub fn head_red(&mut self) -> bool {
		use Term::*;
		match self {
			Num(_) | Var(_) => true,
			Lam(_, b) => b.head_red(),
			App(terms) => match &mut terms[..] {
				[_] => {
					*self = terms.pop().unwrap();
					self.head_red()
				}
				[.., _, Lam(_, _)] => {
					let Some(Lam(v, mut b)) = terms.pop() else {
						unreachable!()
					};
					let Some(a) = terms.pop() else { unreachable!() };

					b.sub(v, a);
					terms.push(*b);

					false
				}
				[.., App(_)] => {
					let Some(App(start)) = terms.pop() else {
						unreachable!()
					};

					terms.extend(start);

					self.head_red()
				}
				_ => true,
			},
		}
	}

	pub fn hnf(&mut self) {
		while !self.head_red() {}
	}

	pub fn hnf_bounded(&mut self, limit: u32) -> bool {
		for _ in 0..limit {
			if self.head_red() {
				return true;
			}
		}
		false
	}

	//A singular left-most reduction. Returns true if in β-nf
	pub fn beta(&mut self) -> bool {
		use Term::*;
		match self {
			Num(_) | Var(_) => true,
			Lam(_, b) => b.beta(),
			App(terms) => match &mut terms[..] {
				[_] => {
					*self = terms.pop().unwrap();
					self.beta()
				}
				[.., _, Lam(_, _)] => {
					let Some(Lam(v, mut b)) = terms.pop() else {
						unreachable!()
					};
					let Some(a) = terms.pop() else { unreachable!() };

					b.sub(v, a);
					terms.push(*b);

					false
				}
				[.., App(_)] => {
					let Some(App(start)) = terms.pop() else {
						unreachable!()
					};

					terms.extend(start);

					self.beta()
				}
				[args @ .., _] => args.iter_mut().rev().all(|arg| arg.beta()),
				[] => unreachable!(),
			},
		}
	}

	pub fn normalize(&mut self) {
		while !self.beta() {}
	}

	pub fn normalize_bounded(&mut self, limit: u32) -> bool {
		for _ in 0..limit {
			if self.beta() {
				return true;
			}
		}
		false
	}

	pub fn free_vars(&self) -> HashSet<Identifier> {
		use Term::*;
		match self {
			Var(x) => HashSet::from([*x]),
			Lam(x, b) => {
				let mut free = b.free_vars();
				free.remove(x);
				free
			}
			App(t) => {
				let mut free = HashSet::new();
				for f in t {
					for v in f.free_vars() {
						free.insert(v);
					}
				}
				free
			}
			_ => HashSet::new(),
		}
	}
}

impl std::fmt::Display for Term {
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Num(k) => write!(fmt, "{}", k),
			Self::Var(v) => write!(fmt, "{}", v),
			Self::Lam(v, b) => write!(fmt, "({}->{})", v, b),
			Self::App(terms) => {
				write!(fmt, "{}", terms.last().unwrap())?;
				for term in terms[..terms.len() - 1].iter().rev() {
					write!(fmt, "({})", term)?;
				}
				Ok(())
			}
		}
	}
}
