use kolmogorov::*;

mod polynomials;
use polynomials::*;

fn main() {
	let lang = PolynomialLanguage;

	let term = term!(f -> plus(one)(f));

	let analysis = lang.analyze(&term);

	println!("{}\n ≈ {}", term, analysis);
}
