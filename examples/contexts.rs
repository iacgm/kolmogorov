use kolmogorov::*;

use Analysis::*;
use Semantics::*;

#[derive(Clone)]
pub struct Polynomials;
impl Language for Polynomials {
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

		let zero = builtin!(
			N
			| | => Num(0)
		);

		let one = builtin!(
			N
			| | => Num(1)
		);

		context! { plus, mult, zero, one }
	}

	fn svar(&self, v: Identifier) -> Analysis {
		Canonical(match v {
			"zero" => SNum(0),
			"one" => SNum(1),
			"plus" | "mult" => SApp(v, vec![]),
			_ => SVar(v),
		})
	}

	fn sapp(&self, fun: Analysis, arg: Analysis) -> Analysis {
		let (fun, arg) = match (fun, arg) {
			(Unique, _) | (_, Unique) => return Unique,
			(Malformed, _) | (_, Malformed) => return Malformed,
			(Canonical(fun), Canonical(arg)) => (fun, arg),
		};

		match (fun, arg) {
			(SApp("plus", _), SNum(0)) => Malformed,
			(SApp("mult", _), SNum(0)) => Malformed,
			(SApp("mult", _), SNum(1)) => Malformed,
			(SApp("plus", mut v), SNum(a)) if !v.is_empty() && matches!(v[0], SNum(_)) => {
				let SNum(b) = v[0] else { unreachable!() };
				v[0] = SNum(a + b);
				Canonical(SApp("plus", v))
			}
			(SApp("mult", mut v), SNum(a)) if !v.is_empty() && matches!(v[0], SNum(_)) => {
				let SNum(b) = v[0] else { unreachable!() };
				v[0] = SNum(a * b);
				Canonical(SApp("mult", v))
			}
			(SApp(f, mut v), arg) => {
				v.push(arg);
				v.sort();
				Canonical(SApp(f, v))
			}
			_ => unreachable!(),
		}
	}
}

#[allow(dead_code)]
fn main() {
	panic!("This file is not intended to be executed directly.")
}
