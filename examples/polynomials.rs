use std::fmt::Display;

use kolmogorov::*;

#[derive(Clone)]
struct PolynomialLanguage;

use Analysis::*;
impl Language for PolynomialLanguage {
	type Semantics = PolySem;

	fn context(&self) -> kolmogorov::Context {
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

	fn snum(&self, n: i32) -> Analysis<Self> {
		Canonical(PolySem::num(n))
	}

	fn svar(&self, v: Identifier) -> Analysis<Self> {
		match v {
			"plus" => Canonical(PolySem(vec![None, None], ))
			v => Canonical(PolySem::var(v))
		}
	}

	fn slam(&self, ident: Identifier, body: Analysis<Self>) -> Analysis<Self> {
		use PolySem::*;
		let Canonical(semantics) = body else {
			return body;
		};

		let semantics = match semantics {
			Func(mut args, body) => {
				args.push(ident);
				Canonical(Func(args, body))
			}
			Prim(, _) => {
				unreachable!()
			}
		};

		Canonical(semantics)
	}

	fn sapp(&self, fun: Analysis<Self>, arg: Analysis<Self>) -> Analysis<Self> {
		let (l, r) = match (fun, arg) {
			(Malformed, _) | (_, Malformed) => return Malformed,
			(Unique, _) | (_, Unique) => return Unique,
			(Canonical(l), Canonical(r)) => (l, r),
		};

		use PolySem::*;
		let canon = match (l, r) {
			(l @ Var("plus") | l @ Var("mult"), r) => App(vec![l, r]),
			(App(vs), b) => match &vs[..] {
				[Var("plus"), _] => vs[1].add(b),
				[Var("mult"), _] => vs[1].add(b),
				_ => unreachable!(),
			},
			(l, r) => unreachable!(">{}|{}", l, r),
		};

		Canonical(canon)
	}
}

impl PolySem {
	pub fn num(n: i32) -> Self {
		Self(vec![], Sum(n, vec![]))
	}

	pub fn var(v: Identifier) -> Self {
		Self(vec![], Sum(0, vec![Product(1, vec![v])]))
	}

	fn mul(self, rhs: Self) -> Self {
		use PolySem::*;
		match (self, rhs) {
			(Num(a), Num(b)) => Num(a * b),
			(Num(n), Var(v)) | (Var(v), Num(n)) => Mul(Product(n, vec![v])),
			(Var(a), Var(b)) => {
				let mut ts = vec![a, b];
				ts.sort();

				Mul(Product(1, ts))
			}
			(Mul(Product(c1, mut ts)), Mul(Product(c2, ts2))) => {
				ts.extend(ts2);
				ts.sort();

				Mul(Product(c1 * c2, ts))
			}
			(Add(Sum(s, mut ts)), Mul(Product(c, ps)))
			| (Mul(Product(c, ps)), Add(Sum(s, mut ts))) => {
				for t in &mut ts {
					t.0 *= c;
					t.1.extend_from_slice(&ps);
					t.1.sort();
				}

				Add(Sum(s, ts))
			}

			_ => unimplemented!(),
		}
	}

	fn add(self, rhs: Self) -> Self {
		use PolySem::*;
		let sum = match (self, rhs) {
			(Num(a), Num(b)) => return Num(a + b),
			(Num(n), Var(v)) | (Var(v), Num(n)) => Sum(n, vec![Product(1, vec![v])]),
			(Var(a), Var(b)) => {
				let mut ts = vec![a, b];
				ts.sort();
				Sum(0, vec![Product(1, ts)])
			}
			(Add(Sum(s1, mut ts1)), Add(Sum(s2, ts2))) => {
				ts1.extend(ts2);
				ts1.sort();

				Sum(s1 + s2, ts1)
			}
			(Add(Sum(s, mut ts)), Mul(p)) | (Mul(p), Add(Sum(s, mut ts))) => {
				ts.push(p);
				ts.sort();

				Sum(s, ts)
			}
			(Mul(p1), Mul(p2)) => Sum(0, vec![p1, p2]),

			_ => unimplemented!(),
		};

		Add(sum)
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
// None indicates an anonymous variable (i.e., one removed by eta-reduction)
struct PolySem(Vec<Identifier>, Sum);

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Sum(i32, Vec<Product>);

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Product(i32, Vec<Identifier>);

impl Semantics for PolySem {}

impl Display for PolySem {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			App(sems) => {
				write!(f, "{}", &sems[0])?;
				for arg in &sems[1..] {
					write!(f, "({})", arg)?
				}
				Ok(())
			}
			Fun(l) => write!(f, "{}", l),
			Add(s) => write!(f, "{}", s),
			Mul(p) => write!(f, "{}", p),
			Num(n) => write!(f, "{}", n),
			Var(v) => write!(f, "{}", v),
		}
	}
}

impl Display for Lambda {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if !self.args.is_empty() {
			write!(f, "\\")?;
			for arg in &self.args {
				write!(f, "{} ", arg)?;
			}
			write!(f, "-> ")?;
		}

		write!(f, "{}", self.body)
	}
}

impl Display for Sum {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)?;
		for b in &self.1 {
			write!(f, "+{}", b)?;
		}
		Ok(())
	}
}

impl Display for Product {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)?;
		for b in &self.1 {
			write!(f, "*{}", b)?;
		}
		Ok(())
	}
}

fn main() {}
