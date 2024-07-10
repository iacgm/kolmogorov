use super::*;
use std::collections::*;
use std::rc::Rc;

type BuiltInFunc = Rc<dyn Fn(&mut Dictionary, &mut [Term]) -> Term>;

#[derive(Clone)]
pub struct BuiltIn {
	pub n_args: usize,
	pub func: BuiltInFunc,
}

#[derive(Clone)]
pub enum Value {
	Term(Term),
	BuiltIn(BuiltIn),
}

type Definition = (Value, Type);

type Entry = Option<Definition>;

pub struct Dictionary {
	defs: HashMap<Identifier, Vec<Entry>>,
}

impl Dictionary {
	//Infers principal type of term
	pub fn infer(&self, term: &Term) -> Option<Type> {
		use Type::*;

		type Defs = HashMap<Identifier, Type>;

		fn core(
			params: (&Dictionary, &mut TypeSub, &mut Defs, &mut VarGen),
			term: &Term,
		) -> Option<Type> {
			let (dict, subs, defs, vgen) = params;
			match term {
				Term::Num(_) => Some(Int),
				Term::Var(v) => {
					if let Some(mono) = defs.get(v) {
						return Some(mono.clone());
					}

					if let Some((_, ty)) = dict.query(v) {
						return Some(ty.clone());
					}

					None
				}
				Term::Lam(v, b) => {
					let newvar = vgen.newvar();

					let old = defs.remove(v);

					let mut tau = Var(newvar);
					defs.insert(v, tau.clone());

					let params = (dict, &mut *subs, &mut *defs, &mut *vgen);
					let out = core(params, b);

					defs.remove(v);

					if let Some(old) = old {
						defs.insert(v, old);
					}

					subs.apply(&mut tau);
					let fun_ty = Fun(tau.into(), out?.into());

					Some(fun_ty)
				}
				Term::App(terms) => {
					let mut terms = terms.iter().rev();

					let fst = terms.next().unwrap();

					let params = (dict, &mut *subs, &mut *defs, &mut *vgen);
					let mut lht = core(params, fst)?;

					for snd in terms {
						let params = (dict, &mut *subs, &mut *defs, &mut *vgen);
						let rht = core(params, snd)?;

						let newvar = vgen.newvar();

						let mut tau = Var(newvar);

						let fun_ty = Fun(rht.into(), tau.clone().into());

						if !subs.unify(&lht, &fun_ty) {
							return None;
						}

						subs.apply(&mut tau);
						lht = tau;
					}

					Some(lht)
				}
			}
		}

		let mut subs = TypeSub::default();
		let mut defs = Defs::default();
		let mut vgen = VarGen::default();

		for (var, entry) in &self.defs {
			vgen.retire(var);
			for (_, t) in entry.iter().filter_map(|o| o.as_ref()) {
				for var in t.vars() {
					vgen.retire(var)
				}
			}
		}

		let params = (self, &mut subs, &mut defs, &mut vgen);
		core(params, term)
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

		match self.query(ident) {
			Some((Value::BuiltIn(BuiltIn { n_args, func }), _)) if *n_args <= n => {
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

	pub fn new(defs: &[(Identifier, Definition)]) -> Self {
		let mut map = HashMap::new();

		for (k, v) in defs {
			map.insert(*k, vec![Some(v.clone())]);
		}

		Self { defs: map }
	}

	pub fn query(&self, ident: Identifier) -> Option<&Definition> {
		let defs = self.defs.get(&ident)?;
		defs.last()?.as_ref()
	}

	pub fn shadow(&mut self, ident: Identifier) {
		let undef = None;
		self.defs.entry(ident).or_default().push(undef);
	}

	pub fn unshadow(&mut self, ident: Identifier) {
		self.defs.get_mut(&ident).unwrap().pop();
	}

	pub fn push_def(&mut self, ident: Identifier, term: Term, ty: Type) {
		let def = Some((Value::Term(term), ty));
		self.defs.entry(ident).or_default().push(def);
	}

	pub fn pop_def(&mut self, ident: Identifier) -> Definition {
		self.defs.get_mut(&ident).unwrap().pop().unwrap().unwrap()
	}
}

impl From<BuiltIn> for Value {
	fn from(value: BuiltIn) -> Self {
		Self::BuiltIn(value)
	}
}

impl From<Term> for Value {
	fn from(value: Term) -> Self {
		Self::Term(value)
	}
}
