use kolmogorov::*;

mod contexts;
use contexts::*;

fn main() {
	let (ctxt, _analyzer) = polynomials();

	let program = term! {
		plus(one)(1)
	};

	println!("{}", ctxt.evaluate(&program));
}
