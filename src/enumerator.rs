//Only works for closed types (i.e., no free variables)
use super::*;

type TermIterator<'a> = Box<dyn Iterator<Item = Term> + 'a>;

pub fn enumerate<'a>(dict: &'a Dictionary, target_ty: &'a Type, size: usize) -> TermIterator<'a> {
	Box::new(vars_producing(dict, target_ty).flat_map(move |(var, ty)| {
		apply_args(dict, target_ty, Stack::from(Term::Var(var)), ty, size - 1)
	}))
}

fn apply_args<'a>(
	dict: &'a Dictionary,
	target_ty: &'a Type,
	lefts: Stack<Term>,
	l_ty: &'a Type,
	r_size: usize,
) -> TermIterator<'a> {
	if l_ty == target_ty {
		if r_size == 0 {
			let apps = lefts.rev_vec();
			let term = Term::App(apps);

			return Box::new(Some(term).into_iter());
		} else {
			return Box::new(std::iter::empty());
		}
	}

	let Type::Fun(d, r_ty) = l_ty else {
		unreachable!();
	};

	let r_ty = &**r_ty;

	Box::new((1..r_size).flat_map(move |arg_size| {
		let lefts = lefts.clone();
		enumerate(dict, d, arg_size).flat_map(move |t| {
			apply_args(dict, target_ty, lefts.cons(t), r_ty, r_size - arg_size - 1)
		})
	}))
}

fn vars_producing<'a>(
	dict: &'a Dictionary,
	ty: &'a Type,
) -> impl Iterator<Item = (Identifier, &'a Type)> + 'a {
	dict.iter_defs().filter_map(move |(&v, def)| match def {
		Def::BuiltIn(_, t) if produces(t, ty) => Some((v, t)),
		_ => None,
	})
}

fn produces(ty: &Type, target: &Type) -> bool {
	target == ty
		|| match ty {
			Type::Fun(_, r) => produces(r, target),
			_ => false,
		}
}
