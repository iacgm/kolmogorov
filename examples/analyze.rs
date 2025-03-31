use kolmogorov::*;

mod polynomials;
use polynomials::*;

fn main() {
	let lang = PolynomialLanguage;

	let term = term!(mult(f)(plus(f)(one)));

	let analysis = lang.analyze(&term);

	println!("{}\n ≈ {}", term, analysis);
}
