use kolmogorov::*;

#[derive(Clone)]
pub struct FibLang;

impl Language for FibLang {
	type Semantics = OpaqueSemantics;

	fn context(&self) -> Context {
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

		context! { lte, plus, minus, one, two}
	}
}

#[allow(dead_code)]
fn main() {
	panic!("This file is not intended to be executed directly.")
}
