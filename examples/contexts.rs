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
			"plus" | "mult" => SVar(v),
			_ => SVar(v),
		})
	}

	fn sapp(&self, fun: Analysis, arg: Analysis) -> Analysis {
		let (fun, arg) = match (fun, arg) {
			(Unique, _) | (_, Unique) => return Unique,
			(Malformed, _) | (_, Malformed) => return Malformed,
			(Canonical(fun), Canonical(arg)) => (fun, arg),
		};

		//println!("In:  {} & {}", fun, arg);

		let fun = match fun {
			SApp(v, mut va) if matches!(&va[..], [SApp(f, _)] if *f == v) => va.remove(0),
			_ => fun,
		};

		//println!("In:  {} & {}", fun, arg);

		let out = match (fun, arg) {
			(SApp("plus", _) | SVar("plus"), SNum(0)) => Malformed,
			(SApp("mult", _) | SVar("mult"), SNum(0)) => Malformed,
			(SApp("mult", _) | SVar("mult"), SNum(1)) => Malformed,

			(SVar(v), arg) => Canonical(SApp(v, vec![arg])),

			(SApp("plus", mut va), SApp("plus", mut vb))
				if matches!((&va[..], &vb[..]), ([SNum(_), ..], [SNum(_), ..])) =>
			{
				let (SNum(a), SNum(b)) = (&va[0], vb.remove(0)) else {
					unreachable!()
				};

				va[0] = SNum(a + b);

				va.extend(vb);
				va.sort();
				Canonical(SApp("plus", va))
			}
			(SApp("mult", mut va), SApp("mult", mut vb))
				if matches!((&va[..], &vb[..]), ([SNum(_), ..], [SNum(_), ..])) =>
			{
				let (SNum(a), SNum(b)) = (&va[0], vb.remove(0)) else {
					unreachable!()
				};

				va[0] = SNum(a * b);

				va.extend(vb);
				va.sort();
				Canonical(SApp("plus", va))
			}
			(SApp("plus", v), SNum(a)) if matches!(&v[..], [SNum(_)]) => {
				let SNum(b) = v[0] else { unreachable!() };
				Canonical(SNum(a + b))
			}
			(SApp("plus", mut v), SNum(a)) if matches!(&v[..], [SNum(_), ..]) => {
				let SNum(b) = v[0] else { unreachable!() };
				v[0] = SNum(a + b);
				Canonical(SApp("plus", v))
			}
			(SApp("mult", v), SNum(a)) if matches!(&v[..], [SNum(_)]) => {
				let SNum(b) = v[0] else { unreachable!() };
				Canonical(SNum(a * b))
			}
			(SApp("mult", mut v), SNum(a)) if matches!(&v[..], [SNum(_), ..]) => {
				let SNum(b) = v[0] else { unreachable!() };
				v[0] = SNum(a * b);
				Canonical(SApp("mult", v))
			}

			(SApp("mult", va), SApp("plus", vb)) => {
				let mut sems = vec![];

				for term in vb {
					let fun = SApp("mult", va.clone());
					let analysis = self.sapp(Canonical(fun), Canonical(term));
					match analysis {
						Canonical(sem) => sems.push(sem),
						_ => return analysis,
					}
				}

				Canonical(SApp("plus", sems))
			}

			(SApp("mult", mut v), arg) if matches!(&v[..], [SApp("plus", _)]) => {
				let SApp("plus", vb) = v.remove(0) else {
					unreachable!()
				};

				let mut sems = vec![];

				let fun = SApp("mult", vec![arg]);
				for term in vb {
					let analysis = self.sapp(Canonical(fun.clone()), Canonical(term));
					match analysis {
						Canonical(sem) => sems.push(sem),
						_ => return analysis,
					}
				}

				Canonical(SApp("plus", sems))
			}

			(SApp(fa, mut va), SApp(fb, vb)) if fa == fb => {
				va.extend(vb);
				va.sort();
				Canonical(SApp(fa, va))
			}

			(SApp(f, mut v), arg) => {
				v.push(arg);
				v.sort();
				Canonical(SApp(f, v))
			}
			_ => unreachable!(),
		};

		//println!("out: {}", out);

		out
	}
}

#[allow(dead_code)]
fn main() {
	panic!("This file is not intended to be executed directly.")
}
