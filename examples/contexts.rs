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

	fn canonize(term: &mut Term) -> bool {
		if let Var(v) = term {
			if *v == "zero" {
				*term = Num(0);
			} else if *v == "one" {
				*term = Num(1);
			}
		}

		if let Some(mut assignment) = term!(plus _ _).unify(term) {
			let mut b = assignment.pop().unwrap();
			let mut a = assignment.pop().unwrap();

			canonize(&mut a);
			canonize(&mut b);

			match (a, b) {
				(Num(a), Num(b)) => {
					*term = Num(a + b);
				}
				(Num(0), t) | (t, Num(0)) => {
					*term = t;
				}
				(Num(a), b) => {
					*term = term!(plus[b][Num(a)]);
					return canonize(term);
				}
				(App(l, r), Num(a)) => {
					if let Num(b) = &*l.borrow() {
						*term = term!(plus[r.borrow()][Num(a + b)]);
					} else if let Num(b) = &*r.borrow() {
						*term = term!(plus[l.borrow()][Num(a + b)]);
					}
				}
				_ => (),
			}
		} else if let Some(mut assignment) = term!(mult _ _).unify(term) {
			let mut b = assignment.pop().unwrap();
			let mut a = assignment.pop().unwrap();

			canonize(&mut a);
			canonize(&mut b);

			match (a, b) {
				(Num(a), Num(b)) => {
					*term = Num(a * b);
				}
				(Num(0), _) | (_, Num(0)) => {
					*term = Num(0);
				}
				(Num(1), t) | (t, Num(1)) => {
					*term = t;
				}
				(Num(a), b) => {
					*term = term!(mult[b][Num(a)]);
					return canonize(term);
				}
				(App(l, r), Num(a)) => {
					if let Num(b) = &*l.borrow() {
						*term = term!(mult[r.borrow()][Num(a + b)]);
					} else if let Num(b) = &*r.borrow() {
						*term = term!(mult[l.borrow()][Num(a + b)]);
					}
				}
				_ => (),
			}
		}

		true
	}

	context! { plus, mult, zero, one % canonize}
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
		if let Var(v) = term {
			if *v == "zero" {
				*term = Num(0);
				return true;
			} else if *v == "one" {
				*term = Num(1);
				return true;
			} else {
				return false;
			}
		}

		if let Some(mut assignment) = term!(plus _ _).unify(term) {
			let mut b = assignment.pop().unwrap();
			let mut a = assignment.pop().unwrap();

			let mut canonized = false;

			canonized |= canonize(&mut a);
			canonized |= canonize(&mut b);

			match (a, b) {
				(Num(a), Num(b)) => {
					*term = Num(a + b);
					canonized = true;
				}
				(App(l, r), Num(a)) | (Num(a), App(l, r)) => {
					if let Num(b) = &*l.borrow() {
						*term = term!(plus[r.borrow()][Num(a + b)]);
						canonized = true;
					} else if let Num(b) = &*r.borrow() {
						*term = term!(plus[l.borrow()][Num(a + b)]);
						canonized = true;
					}
				}
				_ => (),
			}
			return canonized;
		}

		false
	}

	context! { plus, zero, one % canonize }
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
