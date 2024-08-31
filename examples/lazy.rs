use kolmogorov::*;

mod contexts;
use contexts::*;

fn main() {
	let ctxt = polynomials();

	let program = term!{
		plus(one)(1)
	};

	println!("{}", ctxt.evaluate(program));

}
