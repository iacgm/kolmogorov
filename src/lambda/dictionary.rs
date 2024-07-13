use super::*;
use std::collections::*;
use std::rc::Rc;

type BuiltInFunc = Rc<dyn Fn(&mut [Term]) -> Term>;

#[derive(Clone)]
pub struct BuiltIn {
	pub n_args: usize,
	pub func: BuiltInFunc,
}

#[derive(Clone)]
pub enum Def {
	Term(Term),
	BuiltIn(BuiltIn, Type),
}

type Entry = Option<Def>;

pub struct Dictionary {
	defs: HashMap<Identifier, Vec<Entry>>,
}

impl Dictionary {
	pub fn new(defs: &[(Identifier, Def)]) -> Self {
		let mut map = HashMap::new();

		for (k, v) in defs {
			map.insert(*k, vec![Some(v.clone())]);
		}

		Self { defs: map }
	}

	pub fn iter_defs(&self) -> impl Iterator<Item = (&Identifier, &Def)> {
		self.defs.keys().filter_map(|v| Some((v, self.query(v)?)))
	}

	//Requires strong normalization
	pub fn execute(&mut self, term: &mut Term) {
		use Term::*;

		match term {
			Num(_) => (),
			Var(v) => {
				if let Some(Def::Term(t)) = self.query(v) {
					*term = t.clone();
				}
			}
			Lam(v, b) => {
				self.shadow(v);
				self.execute(b);
				self.unshadow(v);
			}
			App(terms) => match &terms[..] {
				[] => unreachable!(),
				[_] => {
					*term = terms.pop().unwrap();
					self.execute(term);
				}
				[.., App(_)] => {
					let App(first) = terms.pop().unwrap() else {
						unreachable!();
					};

					terms.extend(first);
					self.execute(term);
				}
				[.., _, Lam(_, _)] => {
					let Lam(v, mut b) = terms.pop().unwrap() else {
						unreachable!()
					};

					let mut x = terms.pop().unwrap();
					self.execute(&mut x);

					self.push_def(v, Def::Term(x));
					self.execute(&mut b);
					self.pop_def(v);
					terms.push(*b);

					self.execute(term);
				}
				_ => {
					if self.reduce(terms) {
						self.execute(term);
					} else {
						for term in terms {
							self.execute(term);
						}
					}
				}
			},
		}
	}

	//Infers principal type of term
	pub fn infer(&self, term: &Term) -> Option<Type> {
		use Type::*;

		type Args = HashMap<Identifier, Type>;

		fn core(
			params: (&Dictionary, &mut TypeSub, &mut Args, &mut VarGen),
			term: &Term,
		) -> Option<Type> {
			let (dict, subs, defs, vgen) = params;
			match term {
				Term::Num(_) => Some(Int),
				Term::Var(v) => {
					if let Some(mono) = defs.get(v) {
						return Some(mono.clone());
					}

					match dict.query(v)? {
						Def::Term(term) => dict.infer(term),
						Def::BuiltIn(_, ty) => Some(ty.clone()),
					}
				}
				Term::Lam(v, b) => {
					let newvar = vgen.cap_var();

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

						let newvar = vgen.cap_var();

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
		let mut defs = Args::default();
		let mut vgen = VarGen::default();

		for (var, entry) in &self.defs {
			vgen.retire(var);
			for def in entry.iter().filter_map(|o| o.as_ref()) {
				if let Def::BuiltIn(_, t) = def {
					for var in t.vars() {
						vgen.retire(var)
					}
				}
			}
		}

		let params = (self, &mut subs, &mut defs, &mut vgen);
		core(params, term)
	}

	fn reduce(&mut self, terms: &mut Vec<Term>) -> bool {
		use Term::*;

		let ident;
		if let Some(Var(f)) = terms.last() {
			ident = *f;
		} else {
			return false;
		}

		let n = terms.len() - 1;

		match self.query(ident) {
			Some(Def::BuiltIn(BuiltIn { n_args, func }, _)) if *n_args <= n => {
				terms.pop();
				let index = n - n_args;
				let func = func.clone();

				for term in &mut terms[index..] {
					self.execute(term);
				}

				let output = func(&mut terms[index..]);
				terms.truncate(index);
				terms.push(output);
				true
			}
			_ => false,
		}
	}

	fn query(&self, ident: Identifier) -> Option<&Def> {
		let defs = self.defs.get(&ident)?;
		defs.last()?.as_ref()
	}

	fn shadow(&mut self, ident: Identifier) {
		let undef = None;
		self.defs.entry(ident).or_default().push(undef);
	}

	fn unshadow(&mut self, ident: Identifier) {
		self.defs.get_mut(&ident).unwrap().pop();
	}

	fn push_def(&mut self, ident: Identifier, def: Def) {
		let def = Some(def);
		self.defs.entry(ident).or_default().push(def);
	}

	fn pop_def(&mut self, ident: Identifier) -> Def {
		self.defs.get_mut(&ident).unwrap().pop().unwrap().unwrap()
	}
}

impl From<(BuiltIn, Type)> for Def {
	fn from((func, ty): (BuiltIn, Type)) -> Self {
		Self::BuiltIn(func, ty)
	}
}

impl From<Term> for Def {
	fn from(value: Term) -> Self {
		Self::Term(value)
	}
}
