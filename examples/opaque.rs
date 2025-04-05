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
		use Term::*;
		let plus = builtin!(
			N => N => N
			|x, y| => Val(x.int()?+y.int()?)
		);

		let mult = builtin!(
			N => N => N
			|x, y| => Val(x.int()?*y.int()?)
		);

		let one = builtin!(
			N
			| | => Val(1)
		);

		let zero = builtin!(
			N
			| | => Val(0)
		);

		context! { plus, mult, one, zero }
	}

	fn sval(&self, _: i32) -> Analysis<Self> {
		Analysis::Unique
	}

	fn svar(&self, _: Identifier) -> Analysis<Self> {
		Analysis::Unique
	}

	fn sapp(&self, _fun: Analysis<Self>, _arg: Analysis<Self>) -> Analysis<Self> {
		Analysis::Unique
	}

	fn slam(&self, _ident: Identifier, _body: Analysis<Self>) -> Analysis<Self> {
		Analysis::Unique
	}
}

#[allow(unused)]
fn main() {}
