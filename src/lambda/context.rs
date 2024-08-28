//Defines a context (set of typed global variables)
//Distinct from a runtime environment (where variables may
//be untyped & stand for term definitions().

use super::*;
use rustc_hash::FxHashMap as HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct Context {
	defs: HashMap<Identifier, BuiltIn>,
}

impl Context {
	pub fn new(defs: &[(Identifier, BuiltIn)]) -> Self {
		Self {
			defs: HashMap::from_iter(defs.iter().cloned()),
		}
	}

	pub fn insert(&mut self, defs: &[(Identifier, BuiltIn)]) {
		for (ident, def) in defs {
			self.defs.insert(ident, def.clone());
		}
	}

	pub fn iter(&self) -> impl Iterator<Item = (&Identifier, &BuiltIn)> {
		self.defs.iter()
	}

	pub fn get(&self, ident: Identifier) -> Option<&BuiltIn> {
		self.defs.get(ident)
	}

	pub fn vgen(&self) -> VarGen {
		let mut vgen = VarGen::default();

		for var in self.defs.keys() {
			vgen.retire(var);
		}

		vgen
	}

	pub fn vars_producing<'a>(
		&'a self,
		ty: &'a Type,
	) -> impl Iterator<Item = (Identifier, Rc<Type>)> + 'a {
		fn produces(ty: &Type, target: &Type) -> bool {
			let ret_ty_produces = match ty {
				Type::Fun(_, r) => produces(r, target),
				_ => false,
			};

			ret_ty_produces || target == ty
		}

		self.defs
			.iter()
			.filter_map(move |(v, BuiltIn { ty: t, .. })| {
				if produces(t, ty) {
					Some((*v, t.clone()))
				} else {
					None
				}
			})
	}
}
