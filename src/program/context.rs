use super::*;
use std::collections::*;
use std::rc::Rc;

type BuiltIn = dyn Fn(&mut [Term]) -> Term;

#[derive(Clone)]
pub struct ContextEntry {
	pub n_args: usize,
	pub func: Rc<BuiltIn>,
	pub active: bool,
}

pub struct Context {
	dict: HashMap<&'static str, ContextEntry>,
}

impl Context {
	pub fn new(idents: &[(&'static str, ContextEntry)]) -> Self {
		Self {
			dict: HashMap::from_iter(idents.iter().cloned()),
		}
	}

	pub fn set_active(&mut self, ident: &'static str, state: bool) {
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

		let n = terms.len()-1;

		match &self.dict.get(ident) {
			Some(ContextEntry {
				n_args,
				func,
				active,
			}) if *active && *n_args <= n => {			
				terms.pop();
				let index = n - n_args;

				let func = func.clone();

				for arg in &mut terms[index..] {
					arg.exec(self);
				}

				let output = func(&mut terms[index..]);
				terms.truncate(index);
				terms.push(output);
				true
			}
			_ => false,
		}
	}
}
