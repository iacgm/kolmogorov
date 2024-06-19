pub mod parser;
pub use parser::*;

mod types;
use std::collections::HashSet;

type Literal = fn(Term) -> Term;

#[derive(Clone)]
pub enum Term {
	Num(u32),
	Var(&'static str),
	Lit(&'static str, Literal),
	Lam(&'static str, Box<Term>),
	App(Vec<Term>),
}

impl Term {
	pub fn sub(&mut self, var: &'static str, code: Term) {
		use Term::*;
		match self {
			Var(x) if *x == var => *self = code,
			Lam(x, b) => {
				if *x == var {
					let free = code.free_vars();
					let new = new_var_where(|s| s != var && !free.contains(s)).unwrap();
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

	//A singular left-most reduction. Returns true if in β-nf
	pub fn beta(&mut self) -> bool {
		use Term::*;
		match self {
			Num(_) | Var(_) | Lit(_, _) => true,
			Lam(_, b) => b.beta(),
			App(terms) => match &terms[..] {
				[_] => {
					*self = terms.pop().unwrap();
					self.beta()
				}
				[Self::Lam(_, _), _, ..] => {
					let a = terms.remove(1);

					match std::mem::replace(&mut terms[0], Self::Num(0)) {
						Self::Lam(v, mut b) => {
							b.sub(v, a);
							terms[0] = *b;
						}
						_ => unreachable!(),
					}

					if terms.len() == 1 {
						*self = terms.pop().unwrap();
					}
					true
				}
				_ => false,
			},
		}
	}

	pub fn normalize(&mut self, limit: u32) -> bool {
		for i in 0..limit {
			if self.beta() {
				return true;
			}
		}
		false
	}

	pub fn free_vars(&self) -> HashSet<&'static str> {
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
			Self::Lit(s, _) => write!(fmt, "{}", s),
			Self::Lam(v, b) => write!(fmt, "{} -> {}", v, b),
			Self::App(terms) => {
				for term in terms {
					write!(fmt, "({})", term)?;
				}
				Ok(())
			}
		}
	}
}

fn new_var_where(mut p: impl FnMut(&'static str) -> bool) -> Option<&'static str> {
	let options: [&'static str; 26] = [
		"a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r",
		"s", "t", "u", "v", "w", "x", "y", "z",
	];

	options.into_iter().find(|&s| p(s))
}
