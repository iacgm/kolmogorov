use kolmogorov::{
	random::{annotate, random_subnode},
	*,
};

mod polynomials;
use polynomials::*;

fn main() {
	let lang = PolynomialLanguage;

	let term = term!(n -> (f -> plus(n)(n))(one));
	let ty = ty!(N => N);

	let mut immut = ImmutableTerm::from(&term);

	let metadata = annotate(&lang.context(), &immut, &ty, &vec![]);

	random_subnode(&mut immut, &metadata, 1, 2);


}
