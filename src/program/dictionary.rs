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

type Definition = (Value, PolyType);

type Entry = Option<Definition>;

pub struct Dictionary {
	defs: HashMap<Identifier, Vec<Entry>>,
}

impl Dictionary {
	//Infers principal type of term
	pub fn infer(&self, term: &Term) -> Option<PolyType> {
		use MonoType::*;

		type Defs = HashMap<Identifier, MonoType>;

		fn core(params: (&Dictionary, &mut TypeSub, &mut Defs), term: &Term) -> Option<MonoType> {
			let (dict, subs, defs) = params;
			match term {
				Term::Num(_) => Some(Int),
				Term::Var(v) => {
					if let Some(mono) = defs.get(v) {
						return Some(mono.clone());
					}

					if let Some((_, poly)) = dict.query(v) {
						return Some(poly.mono.clone());
					}

					None
				}
				Term::Lam(v, b) => {
					let unused = |v| !dict.contains(v) && defs.values().all(|c| Var(v) != *c);
					let newvar = new_var_where(unused).unwrap();

					let mut tau = Var(newvar);
					defs.insert(v, tau.clone());
					let out = core((dict, subs, defs), b);
					defs.remove(v);

					subs.to_mono(&mut tau);
					let fun_ty = Fun(tau.into(), out?.into());

					Some(fun_ty)
				}
				Term::App(terms) => {
					let mut terms = terms.iter().rev();

					let fst = terms.next().unwrap();

					let mut lht = core((dict, subs, defs), fst)?;

					for snd in terms {
						let rht = core((dict, subs, defs), snd)?;

						let unused = |v| !dict.contains(v) && defs.values().all(|c| Var(v) != *c);
						let newvar = new_var_where(unused).unwrap();

						let mut tau = Var(newvar);

						let fun_ty = Fun(rht.into(), tau.clone().into());

						if !subs.unify(&lht, &fun_ty) {
							return None;
						}

						subs.to_mono(&mut tau);
						lht = tau;
					}

					Some(lht)
				}
			}
		}

		let mut subs = TypeSub::default();
		let mut defs = Default::default();
		let params = (self, &mut subs, &mut defs);
		let poly = core(params, term)?.poly();
		Some(poly)
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

	pub fn newvar(&self) -> Identifier {
		new_var_where(|t| !self.contains(t)).unwrap()
	}

	pub fn contains(&self, ident: Identifier) -> bool {
		if self.defs.contains_key(ident) {
			return true;
		}

		for v in self.defs.values().flatten() {
			match v {
				Some((_, ty)) if ty.contains(ident) => return true,
				Some((Value::Term(t), _)) if t.free_vars().contains(&ident) => return true,
				_ => (),
			}
		}

		false
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

	pub fn push_def(&mut self, ident: Identifier, term: Term, ty: PolyType) {
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
