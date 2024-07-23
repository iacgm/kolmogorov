pub mod dictionary;
pub mod parser;
pub mod types;
pub mod vars;

pub use dictionary::*;
pub use parser::*;
pub use types::*;
pub use vars::*;

use std::collections::HashSet;

#[derive(Clone, Debug)]
pub enum Term {
	Num(i32),
	Var(Identifier),
	Lam(Identifier, Box<Term>),
	//Backwards representation of applications to facilitate
	//pushing & popping from the front
	App(Vec<Term>),
}

impl Term {
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

	pub fn size(&self) -> usize {
		match self {
			Term::Num(_) | Term::Var(_) => 1,
			Term::Lam(_, b) => 1 + b.size(),
			Term::App(terms) => terms.len() - 1 + terms.iter().map(Term::size).sum::<usize>(),
		}
	}

	pub fn applied_to(self, arg: Term) -> Self {
		use Term::*;
		match (self, arg) {
			(App(ls), App(mut rs)) => {
				rs.extend(ls);
				App(rs)
			}
			(lhs, App(mut rs)) => {
				rs.push(lhs);
				App(rs)
			}
			(l, r) => App(vec![r, l]),
		}
	}

	//Convenient shorthand, especially useful for implementing builtins
	pub fn int(&self) -> i32 {
		match self {
			Self::Num(n) => *n,
			_ => unimplemented!(),
		}
	}
}

impl std::fmt::Display for Term {
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use Term::*;
		match self {
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
			App(terms) => {
				write!(fmt, "{}", terms.last().unwrap())?;
				for term in terms[..terms.len() - 1].iter().rev() {
					write!(fmt, "({})", term)?;
				}
				Ok(())
			}
		}
	}
}
