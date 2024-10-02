use kolmogorov::*;

mod contexts;
use contexts::*;

fn main() {
	let lang = Polynomials;

	let term = term!(plus(f)(plus(plus(f)(one))(one)));

	let analysis = lang.analyze(&term);

	println!("{}\n ≈ {}", term, analysis);
}
