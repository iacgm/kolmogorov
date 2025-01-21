use super::*;

use std::fmt::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Analysis {
	Malformed,            // Reject Term entirely (i.e, unnecessarily complex)
	Unique,               // Allow, but did not construct canonical form
	Canonical(Semantics), // Group into equivalence class by canonical form
}

#[derive(Hash, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Semantics {
	SNum(i32),
	SVar(Identifier),
	SLam(Identifier, Box<Semantics>),
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
			Canonical(sem) => Canonical(SLam(ident, sem.into())),
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

	fn analyze(&self, term: &Term) -> Analysis {
		use Term::*;
		match term {
			Num(n) => self.snum(*n),
			Var(v) => self.svar(v),
			Lam(i, b) => self.sabs(i, self.analyze(b)),
			App(l, r) => self.sapp(self.analyze(&l.borrow()), self.analyze(&r.borrow())),
			Ref(r) => self.analyze(&r.borrow()),
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
			SLam(_, _) => {
				let SLam(v, b) = std::mem::replace(self, SNum(0)) else {
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
			SLam(v, b) if *v != var => b.sub(var, def),
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
			Unique => write!(f, "Unique"),
			Malformed => write!(f, "Malformed"),
			Canonical(sem) => write!(f, "Canonical({})", sem),
		}
	}
}

impl Display for Semantics {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		use Semantics::*;
		match self {
			SNum(n) => write!(f, "{}", n),
			SVar(v) => write!(f, "{}", v),
			SLam(v, b) => {
				write!(f, "(\\{}", v)?;
				let mut r = &**b;
				while let SLam(v, next) = r {
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
