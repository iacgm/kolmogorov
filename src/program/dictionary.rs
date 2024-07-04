use super::*;
use std::collections::*;
use std::rc::Rc;

type BuiltIn = Rc<dyn Fn(&mut Dictionary, &mut [Term]) -> Term>;

#[derive(Clone)]
pub struct Definition {
	pub n_args: usize,
	pub func: BuiltIn,
	pub active: bool,
	pub ty: PolyType,
}

pub struct Dictionary {
	defs: HashMap<Identifier, Definition>,
}

impl Dictionary {
	pub fn new(idents: &[(Identifier, Definition)]) -> Self {
		Self {
			defs: HashMap::from_iter(idents.iter().cloned()),
		}
	}
	
	pub fn set_active(&mut self, ident: Identifier, state: bool) {
		if let Some(entry) = self.defs.get_mut(ident) {
			entry.active = state;
		}
	}

	pub fn reduce(&mut self, terms: &mut Vec<Term>) -> bool {
		use Term::*;

		let ident;
		if let Some(Var(f)) = terms.last() {
			ident = *f;
		} else {
			return false;
		}

		let n = terms.len() - 1;

		match &self.defs.get(ident) {
			Some(Definition {
				n_args,
				func,
				active: true,
				..
			}) if *n_args <= n => {
				terms.pop();
				let index = n - n_args;
				let func = func.clone();
				let output = func(self, &mut terms[index..]);
				terms.truncate(index);
				terms.push(output);
				true
			}
			_ => false,
		}
	}
}
