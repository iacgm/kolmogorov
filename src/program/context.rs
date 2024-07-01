use super::*;
use std::collections::*;
use std::rc::Rc;

type BuiltIn = Rc<dyn Fn(&mut Context, &mut [Term]) -> Term>;

#[derive(Clone)]
pub struct ContextEntry {
	pub n_args: usize,
	pub func: BuiltIn,
	pub active: bool,
	pub ty: PolyType,
}

pub struct Context {
	dict: HashMap<Ident, ContextEntry>,
}

impl Context {
	pub fn new(idents: &[(Ident, ContextEntry)]) -> Self {
		Self {
			dict: HashMap::from_iter(idents.iter().cloned()),
		}
	}

	pub fn set_active(&mut self, ident: Ident, state: bool) {
		if let Some(entry) = self.dict.get_mut(ident) {
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

		match &self.dict.get(ident) {
			Some(ContextEntry {
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
