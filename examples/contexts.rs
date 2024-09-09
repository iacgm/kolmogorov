use kolmogorov::*;

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

	let analyzer = std::rc::Rc::new(move |term: &Term| {
		if disallowed_forms
			.iter()
			.any(|form| form.unify(term).is_some())
		{
			Semantics::Malformed
		} else {
			Semantics::Unique
		}
	});

	(context! { plus, mult, zero, one }, Some(analyzer))
}

#[allow(dead_code)]
pub fn fib_ctx() -> (Context, Option<Analyzer>) {
	use Term::*;

	let lte = builtin!(
		N => N => N => N => N
		|a, b| => if a.int()? <= b.int()? {
			term!(a b -> a)
		} else {
			term!(a b -> b)
		}
	);

	let plus = builtin!(
		N => N => N
		|x, y| => Num(x.int()?+y.int()?)
	);

	let minus = builtin!(
		N => N => N
		|x, y| => Num(x.int()?-y.int()?)
	);

	let one = builtin!(
		N
		| | => Num(1)
	);

	let two = builtin!(
		N
		| | => Num(2)
	);

	(context! { lte, plus, minus, one, two }, None)
}

#[allow(dead_code)]
fn main() {
	panic!("This file is not intended to be executed directly.")
}
