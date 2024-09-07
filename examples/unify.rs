use kolmogorov::*;

fn main() {
	let term = term!(plus (plus 0 1) (plus 2 3));

	let unification = term!(plus _ (plus _ _)).unify(&term).unwrap();

	for term in unification.iter() {
		println!("_ {}", term);
	}
}
