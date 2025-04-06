use kolmogorov::{
	metro::{random_subnode, replace_subnode},
	*,
};

mod polynomials;
use polynomials::*;

fn main() {
	let lang = PolynomialLanguage;

	let ctxt = lang.context();

	let start = term!(n -> plus(plus(plus(plus(n)n)n)n)n);

	let ty = ty!(N => N);

	let subnode_tuple = random_subnode(&ctxt, &start, &ty, 1, 5);

	println!("> {:?}", subnode_tuple);
	println!(
		"> {}",
		replace_subnode(&start, subnode_tuple.0, Term::Var("!".into()))
	);
}
