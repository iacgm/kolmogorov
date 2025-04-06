use kolmogorov::*;

mod languages;
use languages::*;

fn main() {
	let lang = Polynomials;

	let term = term!(f -> plus(one)(f));

	let analysis = lang.analyze(&term);

	println!("{}\n ≈ {}", term, analysis);
}
