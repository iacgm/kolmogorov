use kolmogorov::*;

use std::rc::Rc;

#[allow(dead_code)]
pub fn polynomials() -> (Context, Option<Analyzer>) {
	use Term::*;

	let plus = builtin!(
		N => N => N
		|x, y| => Num(x.int()?+y.int()?)
	);

	let mult = builtin!(
		N => N => N
		|x, y| => Num(x.int()?*y.int()?)
	);

	let zero = builtin!(
		N
		| | => Num(0)
	);

	let one = builtin!(
		N
		| | => Num(1)
	);

	let disallowed_forms = [
		term!(plus zero),
		term!(plus _ zero),
		term!(mult zero),
		term!(mult _ zero),
		term!(mult one),
		term!(mult _ one),
	];

	use Analysis::*;
	use std::cell::LazyCell;
	fn canonize(term: &Term, disallowed_forms: &[Term]) -> Analysis {
		if disallowed_forms
			.iter()
			.any(|form| form.unify(term).is_some())
		{
			return Malformed;
		}

		let plus_pat = LazyCell::new(|| term!(plus _ _));
		let mult_pat = LazyCell::new(|| term!(plus _ _));

		use Term::*;
		match term {
			Ref(r) => canonize(&r.borrow(), disallowed_forms),
			Var("zero") => Canonical(Num(0)),
			Var("one") => Canonical(Num(1)),
			Num(_) | Var(_) => Canonical(term.clone()),
			Lam(_, b) => canonize(b, disallowed_forms),
			App(_, _) => {
				if let Some(mut unification) = plus_pat.unify(term) {
					let r = unification.pop().unwrap();
					let l = unification.pop().unwrap();

					let (Canonical(lcan), Canonical(rcan)) = (
						canonize(&l, disallowed_forms),
						canonize(&r, disallowed_forms),
					) else {
						return Unique;
					};

					match (lcan, rcan) {
						(Num(a), Num(b)) => Canonical(Num(a + b)),
						(Num(n), t) | (t, Num(n)) => Canonical(term!(plus[t][Num(n)])),
						_ => Unique,
					}
				} else if let Some(mut unification) = mult_pat.unify(term) {
					let r = unification.pop().unwrap();
					let l = unification.pop().unwrap();

					let (Canonical(lcan), Canonical(rcan)) = (
						canonize(&l, disallowed_forms),
						canonize(&r, disallowed_forms),
					) else {
						return Unique;
					};

					match (lcan, rcan) {
						(Num(a), Num(b)) => Canonical(Num(a * b)),
						(Num(n), t) | (t, Num(n)) => Canonical(term!(mult[t][Num(n)])),
						_ => Unique,
					}
				} else {
					Unique
				}
			}
		}
	}

	let analyzer = Rc::new(move |term: &Term| canonize(term, &disallowed_forms));

	(context! { plus, mult, zero, one }, Some(analyzer))
}

#[allow(dead_code)]
fn main() {
	panic!("This file is not intended to be executed directly.")
}
