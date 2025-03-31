use kolmogorov::*;

mod polynomials;
use polynomials::*;

fn main() {
	let lang = PolynomialLanguage;

	let term = term!(plus(one)(one));

	let analysis = lang.analyze(&term);

	println!("{}\n ≈ {}", term, analysis);
}
