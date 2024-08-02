use super::*;

pub type Identifier = &'static str;

//A few letters removed for legibility (N, omicron, nu, upsilon, )
const IDENTS: &[Identifier] = &[
	"a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s",
	"t", "u", "v", "w", "x", "y", "z", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L",
	"M", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "α", "β", "γ", "δ", "ε", "ζ",
	"η", "θ", "ι", "κ", "λ", "μ", "ξ", "π", "ρ", "ς", "τ", "φ", "χ", "ψ", "ω",
];

//Variable generator
//TODO: Support for infinitely many variables
pub struct VarGen {
	free: HashSet<Identifier>,
}

impl VarGen {
	pub fn newvar(&mut self) -> Identifier {
		let var = *self.free.iter().next().unwrap();
		self.free.take(var).unwrap()
	}

	pub fn small_var(&mut self) -> Identifier {
		self.find_with(|c| c.is_ascii_lowercase())
	}

	pub fn cap_var(&mut self) -> Identifier {
		self.find_with(char::is_ascii_uppercase)
	}

	pub fn find_with(&mut self, p: impl Fn(&char) -> bool) -> Identifier {
		let p = |s: &Identifier| p(&s.chars().next().unwrap());
		let var = self.free.iter().copied().find(p).unwrap();
		self.free.take(var).unwrap()
	}

	pub fn retire(&mut self, ident: Identifier) {
		self.free.remove(ident);
	}

	pub fn freshen(&mut self, ident: Identifier) {
		self.free.insert(ident);
	}
}

impl Default for VarGen {
	fn default() -> Self {
		Self {
			free: HashSet::from_iter(IDENTS.iter().copied()),
		}
	}
}

pub fn new_var_where(mut p: impl FnMut(Identifier) -> bool) -> Option<Identifier> {
	IDENTS.iter().copied().find(|&s| p(s))
}
