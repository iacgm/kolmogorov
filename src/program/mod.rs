pub mod context;
pub mod parser;
pub mod types;

pub use context::*;
pub use parser::*;
pub use types::*;

use std::collections::HashSet;

#[derive(Clone)]
pub enum Term {
	Num(i32),
	Var(&'static str),
	Lam(&'static str, Box<Term>),
	//Backwards representation of applications to facilitate
	//pushing & popping from the front
	App(Vec<Term>),
}

impl Term {
	pub fn exec(&mut self, context: &mut Context) {
		use Term::*;
		loop {
			println!("{}!", self);
			match self {
				Num(_) | Var(_) => break,
				Lam(v, b) => {
					context.set_active(v, false);
					b.exec(context);
					context.set_active(v, true);
					break;
				}
				App(terms) => {
					if let [.., App(_)] = &terms[..] {
						let Some(App(start)) = terms.pop() else {
							unreachable!()
						};

						terms.extend(start);
						continue;
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

					break;
				}
			}
		}
	}

	pub fn sub(&mut self, var: &'static str, code: Term) {
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

	pub fn all_vars(&self) -> HashSet<&'static str> {
		use Term::*;
		match self {
			Var(x) => HashSet::from([*x]),
			Lam(x, b) => {
				let mut vars = b.all_vars();
				vars.insert(x);
				vars
			}
			App(t) => {
				let mut free = HashSet::new();
				for f in t {
					for v in f.all_vars() {
						free.insert(v);
					}
				}
				free
			}
			_ => HashSet::new(),
		}
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

fn new_var_where(mut p: impl FnMut(&'static str) -> bool) -> Option<&'static str> {
	let options = [
		"a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r",
		"s", "t", "u", "v", "w", "x", "y", "z", "α", "β", "γ", "δ", "ε", "ζ", "η", "θ", "ι", "κ",
		"λ", "μ", "ν", "ξ", "ο", "π", "ρ", "ς", "τ", "υ", "φ", "χ", "ψ", "ω",
	];

	options.into_iter().find(|&s| p(s))
}
