use kolmogorov::*;

mod contexts;
use contexts::*;

fn main() {
	let lang = Polynomials;

	let term = term!(plus (plus 1 2));

	let analysis = lang.analyze(&term);

	println!("{}\n ≈ {}", term, analysis);
}
