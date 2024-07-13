use super::*;
use std::collections::HashMap;

type Args = HashMap<Identifier, Type>;

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
	use Term::*;

	fn produces(ty: &Type, target: &Type, subs: &mut TypeSub) -> bool {
		subs.unify(ty, target)
			|| if let Type::Fun(_, r) = ty {
				r.instantiates(target) || produces(r.as_ref(), target, subs)
			} else {
				false
			}
	}

	let (head_var, mut v_ty) = dict.iter_defs().find_map(|(v, def)| match def {
		Def::BuiltIn(_, t) => {
			let fresh = t.fresh(vgen);
			if produces(&fresh, ty, subs) {
				Some((Var(v), fresh))
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

	while !subs.unify(ty, &app_ty) {
		let d = Type::Var(vgen.newvar());
		let r = Type::Var(vgen.newvar());
		let mut fun = ty!([d] => [r]);

		subs.unify(&app_ty, &fun);
		subs.apply(&mut fun);

		let Type::Fun(l, r) = fun else { unreachable!() };

		terms.push(app(&l, size, dict, subs, vgen)?);
		*size -= 1;

		app_ty = *r;
	}

	terms.reverse();

	let term = App(terms);
	println!("<|{}", term);
	Some(term)
}

fn var(ty: &Type, dict: &Dictionary, subs: &mut TypeSub, vgen: &mut VarGen) -> Option<Term> {
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
