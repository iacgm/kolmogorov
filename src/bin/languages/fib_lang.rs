use kolmogorov::*;

#[derive(Clone)]
pub struct FibLang;

impl Language for FibLang {
	type Semantics = OpaqueSemantics;

	fn context(&self) -> Context {
		let int = |t: &Term| t.get::<i32>();

		let lte = builtin!(
			N => N => N => N => N
			|a, b| => if int(&a) <= int(&b) {
				term!(a b -> a)
			} else {
				term!(a b -> b)
			}
		);

		let plus = builtin!(
			N => N => N
			|x, y| => Term::val(int(&x)+int(&y))
		);

		let minus = builtin!(
			N => N => N
			|x, y| => Term::val(int(&x)-int(&y))
		);

		let one = builtin!(
			N
			| | => Term::val(1i32)
		);

		let two = builtin!(
			N
			| | => Term::val(2i32)
		);

		context! { lte, plus, minus, one, two}
	}
}

#[allow(dead_code)]
fn main() {
	panic!("This file is not intended to be executed directly.")
}
