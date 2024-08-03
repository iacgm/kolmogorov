use super::*;
use rustc_hash::FxHashMap as HashMap;
use std::rc::Rc;

type BuiltInFunc = Rc<dyn Fn(&mut [Term]) -> Term>;

#[derive(Clone)]
pub struct BuiltIn {
	pub n_args: usize,
	pub func: BuiltInFunc,
	pub ty: Rc<Type>,
}

#[derive(Clone)]
pub struct Environment {
	ctx: Context,
	defs: HashMap<Identifier, Vec<Option<Term>>>,
}

impl Environment {
	pub fn new(ctx: Context) -> Self {
		Self {
			ctx,
			defs: HashMap::default(),
		}
	}

	pub fn iter_builtins(&self) -> impl Iterator<Item = (&Identifier, &BuiltIn)> {
		self.ctx.iter()
	}

	//Requires strong normalization
	pub fn execute(&mut self, term: &mut Term) {
		use Term::*;

		match term {
			Num(_) => (),
			Var(v) => {
				if let Some(vec) = self.defs.get(v) {
					if let Some(Some(t)) = vec.last() {
						*term = t.clone();
						return;
					}
				}

				if let Some(BuiltIn { func, n_args, .. }) = self.ctx.get(v) {
					if *n_args == 0 {
						*term = func(&mut [])
					}
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

					self.push_def(v, x);
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
			params: (&Environment, &mut TypeSub, &mut Args, &mut VarGen),
			term: &Term,
		) -> Option<Type> {
			let (dict, subs, defs, vgen) = params;
			match term {
				Term::Num(_) => Some(Int),
				Term::Var(v) => {
					if let Some(mono) = defs.get(v) {
						return Some(mono.clone());
					}

					if let Some(vec) = dict.defs.get(v) {
						if let Some(Some(t)) = vec.last() {
							return dict.infer(t);
						}
					}

					let BuiltIn { ty, .. } = dict.ctx.get(v)?;

					Some((**ty).clone())
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

						let tau = Var(newvar);
						let fun_ty = Fun(rht.into(), tau.into());

						match subs.unify(&lht, &fun_ty) {
							Some(Fun(_, tau)) => {
								lht = (*tau).clone();
							}
							_ => return None,
						}
					}

					Some(lht)
				}
			}
		}

		let mut subs = TypeSub::default();
		let mut defs = Args::default();
		let mut vgen = self.vgen();

		let params = (self, &mut subs, &mut defs, &mut vgen);
		core(params, term)
	}

	pub fn vgen(&self) -> VarGen {
		let mut vgen = self.ctx.vgen();

		for var in self.defs.keys() {
			vgen.retire(var);
		}

		vgen
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

		match self.ctx.get(ident) {
			Some(BuiltIn { n_args, func, .. }) if *n_args <= n => {
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

	fn shadow(&mut self, ident: Identifier) {
		let undef = None;
		self.defs.entry(ident).or_default().push(undef);
	}

	fn unshadow(&mut self, ident: Identifier) {
		self.defs.get_mut(&ident).unwrap().pop();
	}

	fn push_def(&mut self, ident: Identifier, def: Term) {
		let def = Some(def);
		self.defs.entry(ident).or_default().push(def);
	}

	fn pop_def(&mut self, ident: Identifier) {
		let _ = self.defs.get_mut(&ident).unwrap().pop();
	}
}
