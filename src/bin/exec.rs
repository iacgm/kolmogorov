use kolmogorov::*;

mod languages;
use languages::*;

fn main() {
	let lang = Polynomials;

	use Term::*;
	let term = App(
		App(Var("plus".into()).into(), Term::val(1).into()).into(),
		Term::val(0).into(),
	);

	println!("out={:?}", term);
	let out = lang.context().evaluate(&term);
	println!("out={}", out);
}
