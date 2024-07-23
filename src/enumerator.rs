//Only works for closed types (i.e., no free variables)
use super::*;

type TermIterator<'a> = Box<dyn Iterator<Item = Term> + 'a>;

static mut COUNTER: usize = 0;

fn alloc<T>(v: T) -> Box<T> {
	unsafe {
		COUNTER += std::mem::size_of_val(&v);
	}
	v.into()
}

pub fn reset_count() {
	unsafe { COUNTER = 0 }
}

pub fn alloc_count() -> usize {
	unsafe { COUNTER }
}

pub fn enumerate<'a>(dict: &'a Dictionary, target_ty: &'a Type, size: usize) -> TermIterator<'a> {
	alloc(vars_producing(dict, target_ty).flat_map(move |(var, ty)| {
		apply_args(dict, target_ty, Stack::one(Term::Var(var)), ty, size - 1)
	}))
}

fn apply_args<'a>(
	dict: &'a Dictionary,
	target_ty: &'a Type,
	lefts: Stack<Term>,
	l_ty: &'a Type,
	r_size: usize,
) -> TermIterator<'a> {
	let done = l_ty == target_ty;

	if r_size == 0 && done {
		let apps = lefts.rev_vec();
		let term = Term::App(apps);

		return alloc(Some(term).into_iter());
	}

	if r_size <= 1 || done {
		return alloc(std::iter::empty());
	}

	let Type::Fun(d, r_ty) = l_ty else {
		unreachable!();
	};

	let r_ty = &**r_ty;

	if r_ty == target_ty {
		return alloc(enumerate(dict, d, r_size - 1).map(move |t| {
			let apps = lefts.cons(t).rev_vec();
			Term::App(apps)
		}));
	}

	alloc((1..r_size).flat_map(move |arg_size| {
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
