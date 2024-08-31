use kolmogorov::*;

mod contexts;
use contexts::*;

fn main() {
	let ctxt = polynomials();

	let program = term!{
		(k -> k)(1)
	};

	println!("{}", ctxt.evaluate(program));

}
