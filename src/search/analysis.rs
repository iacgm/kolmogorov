use super::*;

use std::fmt::*;

#[derive(Clone)]
pub enum Analysis {
	Malformed,            // Reject Term entirely (i.e, unnecessarily complex)
	Unique,               // Allow, but did not construct canonical form
	Canonical(Semantics), // Group into equivalence class by canonical form
}

#[derive(Hash, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Semantics {
	SNum(i32),
	SVar(Identifier),
	SAbs(Identifier, Box<Semantics>),
	SApp(Identifier, Vec<Semantics>),
}

use Analysis::*;
use Semantics::*;
pub trait Language {
	fn context(&self) -> Context;

	fn snum(&self, n: i32) -> Analysis {
		Canonical(SNum(n))
	}
	fn svar(&self, v: Identifier) -> Analysis {
		Canonical(SVar(v))
	}
	fn sabs(&self, ident: Identifier, body: Analysis) -> Analysis {
		match body {
			Malformed => Malformed,
			Unique => Unique,
			Canonical(sem) => Canonical(SAbs(ident, sem.into())),
		}
	}
	fn sapp(&self, fun: Analysis, arg: Analysis) -> Analysis {
		match (fun, arg) {
			(Unique, _) | (_, Unique) => Unique,
			(Malformed, _) | (_, Malformed) => Malformed,
			(Canonical(mut fun), Canonical(arg)) => {
				fun.app(arg);
				Canonical(fun)
			}
		}
	}
}

impl Semantics {
	pub fn app(&mut self, arg: Semantics) {
		use Semantics::*;
		match self {
			SApp(_, args) => {
				args.push(arg);
			}
			SAbs(_, _) => {
				let SAbs(v, b) = std::mem::replace(self, SNum(0)) else {
					unreachable!()
				};
				*self = *b;
				self.sub(v, arg)
			}
			SVar(v) => *self = SApp(v, vec![arg]),
			SNum(_) => unreachable!(),
		}
	}

	pub fn sub(&mut self, var: Identifier, def: Semantics) {
		use Semantics::*;
		match self {
			SVar(v) if *v == var => *self = def,
			SAbs(v, b) if *v != var => b.sub(var, def),
			SApp(v, args) => {
				for arg in args.iter_mut() {
					arg.sub(var, def.clone());
				}
				if *v == var {
					let SApp(_, args) = std::mem::replace(self, def.clone()) else {
						unreachable!()
					};
					for arg in args {
						self.app(arg);
					}
				}
			}
			_ => (),
		}
	}
}

impl Display for Analysis {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		use Analysis::*;
		match self {
			Malformed => write!(f, "Malformed"),
			Unique => write!(f, "Unique"),
			Canonical(term) => write!(f, "Canonical({})", term),
		}
	}
}

impl Display for Semantics {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		use Semantics::*;
		match self {
			SNum(n) => write!(f, "{}", n),
			SVar(v) => write!(f, "{}", v),
			SAbs(v, b) => {
				write!(f, "(\\{}", v)?;
				let mut r = &**b;
				while let SAbs(v, next) = r {
					write!(f, " {}", v)?;
					r = &**next;
				}
				write!(f, " -> {}", r)?;
				write!(f, ")")
			}
			SApp(h, args) => {
				write!(f, "{}", h)?;
				for arg in args {
					write!(f, "({})", arg)?;
				}
				Ok(())
			}
		}
	}
}
