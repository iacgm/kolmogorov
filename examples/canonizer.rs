use kolmogorov::*;

mod contexts;
use contexts::*;

fn main() {
	let (_ctxt, Some(analyzer)) = polynomials() else {
		unreachable!()
	};

	let mut term = term! {
		plus(zero)(plus(one)(f))
	};

	println!("Before: {}", term);
	analyzer(&mut term);
	println!("After : {}", term);
}
