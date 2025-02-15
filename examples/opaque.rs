use std::fmt::Display;

use kolmogorov::*;

#[derive(Clone)]
pub struct Opaque;

impl Language for Opaque {
	type Semantics = OpaqueSemantics;

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

	fn snum(&self, _: i32) -> Analysis<Self> {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OpaqueSemantics;

impl Semantics for OpaqueSemantics {}

impl Display for OpaqueSemantics {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

#[allow(unused)]
fn main() {}
