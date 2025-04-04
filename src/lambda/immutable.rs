use super::*;

use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ImmutableTerm {
	INum(i32),
	IVar(Identifier),
	ILam(Identifier, Rc<ImmutableTerm>),
	IApp(Rc<ImmutableTerm>, Rc<ImmutableTerm>),
}

impl ImmutableTerm {
	pub fn in_beta_normal_form(&self) -> bool {
		
		use ImmutableTerm::*;
		match self {
			INum(_) | IVar(_) => true,
			ILam(_, b) => b.in_beta_normal_form(),
			IApp(l, r) => !matches!(&**l, ILam(_, _)) && l.in_beta_normal_form() && r.in_beta_normal_form(),
		}
	}

	pub fn size(&self) -> usize {
		use ImmutableTerm::*;
		match self {
			INum(_) => 1,
			IVar(_) => 1,
			ILam(_, b) => 1 + b.size(),
			IApp(l, r) => 1 + l.size() + r.size(),
		}
	}

	pub fn term(&self) -> Term {
		use ImmutableTerm::*;
		use Term::*;

		match self {
			INum(n) => Num(*n),
			IVar(v) => Var(*v),
			ILam(v, b) => Lam(*v, b.term().into()),
			IApp(l, r) => App(l.term().into(), r.term().into()),
		}
	}
}

impl From<&Term> for ImmutableTerm {
	fn from(term: &Term) -> Self {
		use ImmutableTerm::*;
		use Term::*;

		match term {
			Ref(r) => (&*r.borrow()).into(),
			Num(n) => INum(*n),
			Var(v) => IVar(*v),
			Lam(v, b) => ILam(*v, Self::from(&**b).into()),
			App(l, r) => IApp(
				Self::from(&*l.borrow()).into(),
				Self::from(&*r.borrow()).into(),
			),
		}
	}
}

use std::fmt::*;
impl Display for ImmutableTerm {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
		use ImmutableTerm::*;
		match self {
			INum(k) => write!(fmt, "{}", k),
			IVar(v) => write!(fmt, "{}", v),
			ILam(v, b) => {
				write!(fmt, "(\\{}", v)?;
				let mut r = &**b;
				while let ILam(v, next) = r {
					write!(fmt, " {}", v)?;
					r = &**next;
				}
				write!(fmt, " -> {}", r)?;
				write!(fmt, ")")
			}
			IApp(l, r) => {
				write!(fmt, "{}({})", &**l, &**r)
			}
		}
	}
}
