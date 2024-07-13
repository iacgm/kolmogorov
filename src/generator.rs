use super::*;

use rand::prelude::*;

//generates a term of a given type, with a specified size
pub fn generate_term(dict: &Dictionary, ty: &Type, mut size: usize) -> Option<Term> {
	let mut subs = TypeSub::default();
	let mut vgen = VarGen::default();

	app(ty, &mut size, dict, &mut subs, &mut vgen)
}

fn app(
	ty: &Type,
	size: &mut usize,
	dict: &Dictionary,
	subs: &mut TypeSub,
	vgen: &mut VarGen,
) -> Option<Term> {
	use Type::*;

	fn produces(ty: &Type, target: &Type, subs: &mut TypeSub) -> bool {
		subs.unify(ty, target).is_some()
			|| if let Fun(_, r) = ty {
				r.instantiates(target) || produces(r.as_ref(), target, subs)
			} else {
				false
			}
	}

	let mut defs: Vec<_> = dict.iter_defs().collect();

	defs.shuffle(&mut thread_rng());

	let (head_var, mut v_ty) = defs
		.iter()
		.find_map(|(v, def)| match def {
			Def::BuiltIn(_, t) => {
				let fresh = t.fresh(vgen);
				if produces(&fresh, ty, subs) {
					Some((Term::Var(v), fresh))
				} else {
					for v in fresh.vars() {
						vgen.retire(v);
					}
					None
				}
			}
			Def::Term(_) => None,
		})?;

	subs.apply(&mut v_ty);

	let mut terms = vec![head_var];
	let mut app_ty = v_ty;

	loop {
		if subs.unify(ty, &app_ty).is_some() {
			terms.reverse();
			return Some(Term::App(terms));
		}

		let d = vgen.newvar();
		let r = vgen.newvar();
		let Some(Fun(l, r)) = subs.unify(&app_ty, &ty!([Var(d)] => [Var(r)])) else {
			unreachable!()
		};

		terms.push(app(&l, size, dict, subs, vgen)?);
		app_ty = *r;
	}
}

/* fn var(ty: &Type, dict: &Dictionary, subs: &mut TypeSub, vgen: &mut VarGen) -> Option<Term> {
	use Term::*;
	dict.iter_defs().find_map(|(v, def)| match def {
		Def::BuiltIn(_, t) => {
			let fresh = t.fresh(vgen);
			if subs.unify(&fresh, ty) {
				Some(Var(v))
			} else {
				for v in fresh.vars() {
					vgen.retire(v);
				}
				None
			}
		}
		Def::Term(_) => None,
	})
}
 */
