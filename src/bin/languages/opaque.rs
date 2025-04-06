use kolmogorov::*;

#[derive(Clone)]
pub struct Opaque;

impl Language for Opaque {
	type Semantics = OpaqueSemantics;

	// Max size of `small` terms. (TODO: Make language-dependent)
	const SMALL_SIZE: usize = 5;

	// Max size of `large` terms. (TODO: Make language-dependent)
	const LARGE_SIZE: usize = 8;

	fn context(&self) -> Context {
		let int = |t: &Term| t.get::<i32>();

		let plus = builtin!(
			N => N => N
			|x, y| => Term::val(int(&x)?+int(&y)?)
		);

		let mult = builtin!(
			N => N => N
			|x, y| => Term::val(int(&x)?*int(&y)?)
		);

		let one = builtin!(
			N
			| | => Term::val(1)
		);

		let zero = builtin!(
			N
			| | => Term::val(0)
		);

		context! { plus, mult, one, zero }
	}
}

#[allow(unused)]
fn main() {}
