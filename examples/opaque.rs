use kolmogorov::*;

#[derive(Clone)]
pub struct Opaque;

impl Language for Opaque {
	type Semantics = ();

	fn context(&self) -> Context {
		use Term::*;
		let pads = builtin!(
			N => N
			|x| => x.clone()
		);

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

		context! { pads, plus, mult, one, zero }
	}
}

impl Semantics for () {
	
}

#[allow(unused)]
fn main() {}
