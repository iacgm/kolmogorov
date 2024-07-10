use super::*;

pub type Identifier = &'static str;

const IDENTS: [Identifier; 76] = [
	"a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s",
	"t", "u", "v", "w", "x", "y", "z", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L",
	"M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "α", "β", "γ", "δ", "ε",
	"ζ", "η", "θ", "ι", "κ", "λ", "μ", "ν", "ξ", "ο", "π", "ρ", "ς", "τ", "υ", "φ", "χ", "ψ", "ω",
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
			free: HashSet::from(IDENTS),
		}
	}
}

pub fn new_var_where(mut p: impl FnMut(Identifier) -> bool) -> Option<Identifier> {
	IDENTS.into_iter().find(|&s| p(s))
}
