pub mod parser;
pub use parser::*;

mod types;
use std::collections::HashSet;
use std::rc::Rc;

type Literal = Rc<dyn Fn(&mut Vec<Term>) -> bool>;

#[derive(Clone)]
pub enum Term {
	Num(i32),
	Var(&'static str),
	Nam(&'static str, Box<Term>),
	Lit(&'static str, Literal),
	Lam(&'static str, Box<Term>),
	//Backwards representation of applications to facilitate
	//pushing & popping from the front
	App(Vec<Term>),
}

impl Term {
	pub fn sub(&mut self, var: &'static str, code: Term) {
		use Term::*;
		match self {
			Var(x) if *x == var => *self = code,
			Nam(_, b) => b.sub(var, code),
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
			Nam(_, b) | Lam(_, b) => b.beta(),
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
				[.., Lit(_, _)] => {
					let Some(Lit(_, transform)) = terms.pop() else {
						unreachable!()
					};

					transform(terms)
				}
				[.., Nam(_, _)] => {
					let Some(Nam(_, term)) = terms.pop() else {
						unreachable!()
					};

					terms.push(*term);

					false
				}
				[.., App(_)] => {
					let Some(App(start)) = terms.pop() else {
						unreachable!()
					};

					terms.extend(start);

					false
				}
				[args @ .., _] => args.iter_mut().rev().all(|arg| arg.beta()),
				[] => unreachable!(),
			},
		}
	}

	pub fn expand(&mut self) {
		match self {
			Self::Nam(_, boxed) => {
				*self = std::mem::replace(boxed.as_mut(), Self::Num(0));
			},
			Self::App(terms) => terms.iter_mut().for_each(|term| term.expand()),
			Self::Lam(_, b) => b.expand(),
			_ => (),
		}
	}

	pub fn normalize(&mut self) {
		while !self.beta() {}
	}

	pub fn bounded_normalize(&mut self, limit: u32) -> bool {
		for _ in 0..limit {
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
			Self::Nam(name, _) => write!(fmt, "{}", name),
			Self::Var(v) => write!(fmt, "{}", v),
			Self::Lit(s, _) => write!(fmt, "{}", s),
			Self::Lam(v, b) => write!(fmt, "({}->{})", v, b),
			Self::App(terms) => {
				write!(fmt, "{}", terms.last().unwrap())?;
				for term in terms[..terms.len()-1].iter().rev() {
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
