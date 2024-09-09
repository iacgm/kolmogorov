use kolmogorov::*;

mod contexts;
use contexts::*;

fn main() {
	let (_ctxt, Some(analyze)) = polynomials() else {
		unreachable!()
	};

	let term = term! {
		plus(f)(plus(one)(f))
	};

	println!("Term : {}", term);
	println!("Canon: {}", analyze(&term));
}
