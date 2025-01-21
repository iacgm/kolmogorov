use kolmogorov::*;

#[derive(Clone)]
pub struct Opaque;

impl Language for Opaque {
	fn context(&self) -> Context {
		use Term::*;
		let plus = builtin!(
			N => N => N
			|x, y| => Num(x.int()?+y.int()?)
		);

		let mult = builtin!(
			N => N => N
			|x, y| => Num(x.int()?*y.int()?)
		);

		let one = builtin!(
			N
			| | => Num(1)
		);

		let zero = builtin!(
			N
			| | => Num(0)
		);

		context! { plus, mult, one, zero }
	}
}
