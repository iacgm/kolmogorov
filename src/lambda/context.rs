//Defines a context (set of typed global variables)
//Distinct from a runtime environment (where variables may
//be untyped & stand for term definitions().

use super::*;
use rustc_hash::FxHashMap as HashMap;

#[derive(Clone)]
pub struct Context {
	ctx: HashMap<Identifier, BuiltIn>,
}

impl Context {
	pub fn new(defs: &[(Identifier, BuiltIn)]) -> Self {
		Self {
			ctx: HashMap::from_iter(defs.iter().cloned()),
		}
	}

	pub fn iter(&self) -> impl Iterator<Item = (&Identifier, &BuiltIn)> {
		self.ctx.iter()
	}

	pub fn get(&self, ident: Identifier) -> Option<&BuiltIn> {
		self.ctx.get(ident)
	}

	pub fn vgen(&self) -> VarGen {
		let mut vgen = VarGen::default();

		for var in self.ctx.keys() {
			vgen.retire(var);
		}

		vgen
	}
}
