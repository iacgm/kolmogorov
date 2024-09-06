use kolmogorov::*;

mod contexts;
use contexts::*;

fn main() {
	let ctxt = polynomials();

	let mut term = term!{
		plus(zero)(plus(one)(f))
	}; 

	println!("Before: {}", term);
	(*ctxt.canonizer)(&mut term);
	println!("After : {}", term);
}
