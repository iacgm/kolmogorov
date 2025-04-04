// Third iteration of search implementation, which uses Boxed nodes
// Ideally will be simpler than the stack machine & as fast as the
// iterator enumerator, (while making caching & other optimizations
// easier to implement & maintain)

mod analysis;
mod cache;
mod node;
mod semantics;

pub use analysis::*;
pub use semantics::*;

use super::*;
use cache::*;
use node::*;

use std::rc::Rc;

pub fn search<'a, L: Language>(
	lang: &'a L,
	vars: VarsVec,
	targ: &Type,
	size: usize,
) -> Enumerator<'a, L> {
	let ctxt = lang.context();

	let mut vgen = ctxt.vgen();

	for (var, _) in &vars {
		vgen.retire(*var);
	}

	Enumerator {
		search_ctxt: SearchContext {
			lang,
			ctxt,
			vgen,
			args: vars,
			cache: Cache::new(),
		},
		root: Node::All {
			targ: Rc::new(targ.clone()),
			size,
			state: None,
			phase: AllPhase::START,
		},
	}
}

pub struct Enumerator<'a, L: Language> {
	search_ctxt: SearchContext<'a, L>,
	root: Node<L>,
}

pub type VarDecl = (Identifier, Rc<Type>);
pub type VarsVec = Vec<VarDecl>;

struct SearchContext<'a, L: Language> {
	lang: &'a L,
	ctxt: Context,
	vgen: VarGen,
	// Variables from abstractions
	args: VarsVec,
	cache: Cache<L>,
}

impl<'a, L: Language> SearchContext<'a, L> {
	fn contains_var_of_type(&self, ty: &Rc<Type>) -> bool {
		let args = self.args.iter().map(|(_, t)| t);
		let ctxt = self.ctxt.iter().map(|(_, b)| &b.ty);

		args.chain(ctxt).any(|v_ty| v_ty == ty)
	}

	fn vars_producing(&mut self, targ: &Rc<Type>) -> VarsVec {
		fn produces(ty: &Type, target: &Type) -> bool {
			let ret_ty_produces = match ty {
				Type::Fun(_, r) => produces(r, target),
				_ => false,
			};

			ret_ty_produces || target == ty
		}

		let var_produces = move |(v, ty): (Identifier, &Rc<Type>)| {
			if produces(ty, targ) {
				Some((v, ty.clone()))
			} else {
				None
			}
		};

		let vars = self
			.ctxt
			.iter()
			.map(|(&v, BuiltIn { ty, .. })| (v, ty))
			.chain(self.args.iter().map(|(v, t)| (*v, t)))
			.filter_map(var_produces)
			.collect();

		vars
	}
}

impl<'a, L: Language> Iterator for Enumerator<'a, L> {
	type Item = (ImmutableTerm, Analysis<L>);

	fn next(&mut self) -> Option<Self::Item> {
		self.root.next(&mut self.search_ctxt)
	}
}
