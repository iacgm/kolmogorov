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

	pub fn evaluate(&self, term: &Term) -> Term {
		let mut thunk: Thunk = term.clone().into();
		self.evaluate_thunk(&mut thunk);
		Rc::unwrap_or_clone(thunk).into_inner()
	}

	// True if any work done
	pub fn evaluate_thunk(&self, thunk: &mut Thunk) {
		use Term::*;
		let mut borrow = (**thunk).borrow_mut();
		let term = &mut *borrow;
		match term {
			Num(_) | Lam(_, _) => (),
			Var(v) => {
				if let Some(BuiltIn {
					func, n_args: 0, ..
				}) = self.get(v)
				{
					*term = func(&mut []).unwrap();
					drop(borrow);
					self.evaluate_thunk(thunk)
				}
			}
			Ref(next) => {
				let next = next.clone();
				drop(borrow);
				*thunk = next;
				self.evaluate_thunk(thunk)
			}
			App(_, _) => {
				self.collapse_spine(term, 0);
			}
		}
	}

	fn collapse_spine(&self, root: &mut Term, depth: usize) -> SpineCollapse {
		use SpineCollapse::*;
		use Term::*;
		match root {
			Ref(thunk) => self.collapse_spine(&mut thunk.borrow_mut(), depth),
			Num(_) | Lam(_, _) => Whnf,
			Var(v) => match self.get(v) {
				Some(BuiltIn {
					func, n_args: 0, ..
				}) => {
					*root = func(&mut []).unwrap();
					self.collapse_spine(root, depth)
				}
				Some(blt) if blt.n_args <= depth => {
					Exec(blt.clone(), Vec::with_capacity(blt.n_args))
				}
				_ => Whnf,
			},
			App(l, r) => {
				let mut borr = l.borrow_mut();
				match self.collapse_spine(&mut borr, depth + 1) {
					Exec(builtin, mut args) => {
						drop(borr);
						let argc = builtin.n_args;

						args.push(r.clone());

						if argc == args.len() {
							for arg in &mut args[..] {
								self.evaluate_thunk(arg);
							}

							let func = &*builtin.func;

							if let Some(term) = func(&mut args[..]) {
								*root = term;
								return self.collapse_spine(root, depth);
							}
						}

						Exec(builtin, args)
					}
					Whnf => {
						drop(borr);
						let borr = (**l).borrow();
						if let Lam(_, _) = *borr {
							let Lam(v, b) = borr.clone() else {
								unreachable!()
							};

							drop(borr);

							*root = b.instantiate_var(v, r);
							self.collapse_spine(root, depth)
						} else {
							drop(borr);
							Whnf
						}
					}
				}
			}
		}
	}
}

enum SpineCollapse {
	Whnf,
	Exec(BuiltIn, Vec<Thunk>),
}
