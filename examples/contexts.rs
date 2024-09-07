use kolmogorov::*;

#[allow(dead_code)]
pub fn polynomials() -> Context {
	use Term::*;

	let plus = builtin!(
		N => N => N
		|x, y| => Num(x.int()?+y.int()?)
	);

	let mult = builtin!(
		N => N => N
		|x, y| => {
			if *x == Term::Num(0) || *y == Term::Num(0) {
				Num(0)
			} else {
				Num(x.int()?*y.int()?)
			}
		}
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

	let validate = move |term: &Term| {
		disallowed_forms
			.iter()
			.all(|form| form.unify(term).is_none())
	};

	context! { plus, mult, zero, one & validate}
}

#[allow(dead_code)]
pub fn sums() -> Context {
	use Term::*;

	let plus = builtin!(
		N => N => N
		|x, y| => Num(x.int()?+y.int()?)
	);

	let zero = builtin!(
		N
		| | => Num(0)
	);

	let one = builtin!(
		N
		| | => Num(1)
	);

	fn canonize(term: &mut Term) -> bool {
		match term {
			Ref(r) => {
				canonize(&mut r.borrow_mut());
			}
			Var("zero") => {
				*term = Num(0);
				return true;
			}
			Var("one") => {
				*term = Num(1);
				return true;
			}
			App(l, r) => {
				canonize(&mut l.borrow_mut());
				canonize(&mut r.borrow_mut());
			}
			_ => (),
		}

		if let Some(mut assignment) = term!(plus _ _).unify(term) {
			let b = assignment.pop().unwrap();
			let a = assignment.pop().unwrap();

			match (a, b) {
				(Num(a), Num(b)) => {
					*term = Num(a + b);
				}
				(Num(0), t) | (t, Num(0)) => *term = t,
				(App(l, r), Num(a)) | (Num(a), App(l, r)) => {
					if let Num(b) = &*l.borrow() {
						*term = term!(plus[r.borrow()][Num(a + b)]);
					} else if let Num(b) = &*r.borrow() {
						*term = term!(plus[l.borrow()][Num(a + b)]);
					}
				}
				_ => (),
			}
		}

		true
	}

	context! { plus, zero, one  }
}

#[allow(dead_code)]
pub fn fib_ctx() -> Context {
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

	context! { lte, plus, minus, one, two }
}

#[allow(dead_code)]
fn main() {
	panic!("This file is not intended to be executed directly.")
}
