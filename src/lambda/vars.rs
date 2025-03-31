use std::fmt::{Debug, Display};

use super::*;

#[derive(Hash, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub enum Identifier {
	Name(&'static str),
	Uuid(u128),
}

//A few letters removed for legibility (N, omicron, nu, upsilon, )
const IDENTS: &[&str] = &[
	"a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s",
	"t", "u", "v", "w", "x", "y", "z", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L",
	"M", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "α", "β", "γ", "δ", "ε", "ζ",
	"η", "θ", "ι", "κ", "λ", "μ", "ξ", "π", "ρ", "ς", "τ", "φ", "χ", "ψ", "ω",
];

//Variable generator
pub struct VarGen {
	free: HashSet<Identifier>,
}

impl VarGen {
	pub fn newvar(&mut self) -> Identifier {
		let var = *self.free.iter().next().unwrap();
		self.free.take(&var).unwrap()
	}

	pub fn small_var(&mut self) -> Identifier {
		self.find_with(|c| c.is_ascii_lowercase())
	}

	pub fn cap_var(&mut self) -> Identifier {
		self.find_with(char::is_ascii_uppercase)
	}

	pub fn find_with(&mut self, p: impl Fn(&char) -> bool) -> Identifier {
		let p = |s: &Identifier| p(&s.str().chars().next().unwrap());
		let var = self.free.iter().copied().find(p).unwrap();
		self.free.take(&var).unwrap()
	}

	pub fn retire(&mut self, ident: Identifier) {
		self.free.remove(&ident);
	}

	pub fn freshen(&mut self, ident: Identifier) {
		self.free.insert(ident);
	}
}

impl Default for VarGen {
	fn default() -> Self {
		Self {
			free: HashSet::from_iter(IDENTS.iter().copied().map(Identifier::Name)),
		}
	}
}

pub fn new_var_where(mut p: impl FnMut(Identifier) -> bool) -> Option<Identifier> {
	IDENTS
		.iter()
		.copied()
		.map(Identifier::Name)
		.find(|&id| p(id))
}

pub fn uuid() -> Identifier {
	// Technically unsafe in a multithreaded setting, but I'll be shocked if this isn't fine forever.

	static mut COUNTER: u128 = 0;

	let x;
	unsafe {
		x = COUNTER;
		COUNTER += 1;
	}

	Identifier::Uuid(x)
}

impl Display for Identifier {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use Identifier::*;
		match self {
			Name(s) => write!(f, "{}", s),
			Uuid(u) => write!(f, "_{}", u),
		}
	}
}

impl Debug for Identifier {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self)
	}
}

impl From<&'static str> for Identifier {
	fn from(s: &'static str) -> Self {
		Self::Name(s)
	}
}

impl From<u128> for Identifier {
	fn from(u: u128) -> Self {
		Self::Uuid(u)
	}
}

impl Identifier {
	pub fn str(&self) -> &'static str {
		match self {
			Self::Name(s) => s,
			_ => unimplemented!(),
		}
	}
}
