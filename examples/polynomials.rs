use std::fmt::Display;

use kolmogorov::*;

#[derive(Clone)]
pub struct PolynomialLanguage;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PolySem(Vec<Identifier>, Sum); // Last arg is first to be applied. (outermost last)

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Sum(i32, Vec<Product>);

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Product(i32, Vec<Identifier>);

use Analysis::*;
impl Language for PolynomialLanguage {
	type Semantics = PolySem;

	const SMALL_SIZE: usize = 10;
	const LARGE_SIZE: usize = 20;

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
		use Identifier::*;
		match v {
			Name("plus") => {
				let a = Uuid(0);
				let b = Uuid(1);
				Canonical(PolySem(vec![a, b], Sum::from([a, b])))
			}
			Name("mult") => {
				let a = Uuid(0);
				let b = Uuid(1);
				Canonical(PolySem(vec![a, b], Product::from([a, b]).into()))
			}
			Name("zero") => Canonical(PolySem::num(0)),
			Name("one") => Canonical(PolySem::num(1)),
			v => Canonical(PolySem::var(v)),
		}
	}

	fn slam(&self, ident: Identifier, body: Analysis<Self>) -> Analysis<Self> {
		let mut body_sem = match body {
			Canonical(c) => c,
			_ => return body,
		};

		body_sem.0.push(ident);

		Canonical(body_sem)
	}

	fn sapp(&self, fun: Analysis<Self>, arg: Analysis<Self>) -> Analysis<Self> {
		let fun_sem = match fun {
			Canonical(c) => c,
			_ => return fun,
		};

		let arg_sem = match arg {
			Canonical(c) => c,
			_ => return arg,
		};

		let PolySem(vs, sum) = arg_sem;

		assert!(vs.is_empty());

		Canonical(fun_sem.apply_to_sum(sum))
	}
}

impl PolySem {
	pub fn num(n: i32) -> Self {
		Self(vec![], Sum::from(n))
	}

	pub fn var(v: Identifier) -> Self {
		Self(vec![], v.into())
	}

	fn apply_to_sum(self, sum: Sum) -> Self {
		let PolySem(mut args, body) = self;
		let ident = args.pop().expect("Did not expect argument.");
		let Sum(shift, ps) = body;

		let mut output = Sum(shift, vec![]);

		for Product(s1, vs1) in ps {
			let mut prod_terms = Product(s1, vec![]);
			let mut count = 0;

			for var in vs1 {
				if var != ident {
					prod_terms.1.push(var);
				} else {
					count += 1;
				}
			}

			let pow = sum.to_pow(count);
			let addends = pow.mul_prod(&prod_terms);

			output = output.add(&addends);
		}

		output.normalize();

		PolySem(args, output)
	}
}

impl Sum {
	// Ensure everything is properly structured
	pub fn normalize(&mut self) {
		use std::collections::hash_map::*;
		let Sum(c, ps) = self;

		// Remove empty products
		let mut i = 0;
		while i < ps.len() {
			let Product(s, vs) = &mut ps[i];

			if vs.is_empty() {
				*c += *s;
				ps.swap_remove(i);
				continue;
			}
			i += 1;
		}

		let mut map = HashMap::new();

		for Product(s, mut vs) in ps.drain(..) {
			vs.sort();

			match map.entry(vs) {
				Entry::Vacant(e) => {
					e.insert(s);
				}
				Entry::Occupied(mut v) => *v.get_mut() += s,
			}
		}

		*ps = map.into_iter().map(|(vs, c)| Product(c, vs)).collect();
		ps.sort();
	}

	pub fn to_pow(&self, n: usize) -> Sum {
		if n == 0 {
			Sum::from(1)
		} else if n % 2 == 0 {
			let a = &self.to_pow(n / 2);
			a.mul(a)
		} else {
			self.mul(&self.to_pow(n - 1))
		}
	}

	pub fn mul_prod(&self, rhs: &Product) -> Sum {
		let Sum(c, ps) = self;
		let Product(s, vs) = rhs;

		if vs.is_empty() {
			return Sum(
				c * s,
				ps.iter()
					.map(|Product(k, ps)| Product(k * s, ps.clone()))
					.collect(),
			);
		}

		let mut terms = vec![];

		if *c != 0 {
			terms.push(Product(c * s, vs.clone()));
		}

		for prod in ps {
			terms.push(prod.mul(rhs))
		}

		Sum(0, terms)
	}

	pub fn mul(&self, rhs: &Sum) -> Sum {
		let Sum(c1, ps1) = self;
		let Sum(c2, ps2) = rhs;

		let mut terms = vec![];

		if *c1 != 0 {
			for Product(s, vs) in ps2 {
				terms.push(Product(*c1 * s, vs.clone()));
			}
		}

		if *c2 != 0 {
			for Product(s, vs) in ps1 {
				terms.push(Product(*c2 * s, vs.clone()));
			}
		}

		for p1 in ps1 {
			for p2 in ps2 {
				terms.push(p1.mul(p2))
			}
		}

		Sum(c1 * c2, terms)
	}

	pub fn add(&self, rhs: &Sum) -> Sum {
		let Sum(c1, ps1) = self;
		let Sum(c2, ps2) = rhs;

		let mut ps = ps1.clone();
		ps.extend_from_slice(&ps2[..]);

		Sum(c1 + c2, ps)
	}
}

impl Product {
	pub fn mul(&self, rhs: &Product) -> Product {
		let Product(c1, vs1) = self;
		let Product(c2, vs2) = rhs;

		if *c1 == 0 || *c2 == 0 {
			return Product(0, vec![]);
		}

		let mut vs = vs1.clone();
		vs.extend_from_slice(&vs2[..]);

		Product(c1 * c2, vs)
	}
}

impl Semantics for PolySem {}

impl From<Product> for Sum {
	fn from(prod: Product) -> Self {
		Self(0, vec![prod])
	}
}

impl From<Identifier> for Sum {
	fn from(id: Identifier) -> Self {
		Self(0, vec![Product::from(id)])
	}
}

impl<const N: usize> From<[Identifier; N]> for Sum {
	fn from(ids: [Identifier; N]) -> Self {
		Self(0, ids.into_iter().map(Product::from).collect())
	}
}

impl From<i32> for Sum {
	fn from(prod: i32) -> Self {
		Self(prod, vec![])
	}
}

impl From<Identifier> for Product {
	fn from(id: Identifier) -> Self {
		Self(1, vec![id])
	}
}
impl<const N: usize> From<[Identifier; N]> for Product {
	fn from(ids: [Identifier; N]) -> Self {
		Self(1, ids.to_vec())
	}
}

impl Display for PolySem {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?} -> {}", self.0, self.1)
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

#[allow(dead_code)]
fn main() {
	panic!("This file is not intended to be executed directly.")
}
