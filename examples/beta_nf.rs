use kolmogorov::*;

fn main() {
	let term = term!(((k -> mult(k)(plus(k)(k)))(plus(f)(one)))(plus(zero)(f)));

	let immut = ImmutableTerm::from(&term);

	let b_nf = immut.in_beta_normal_form();

	println!();
	println!("> {}", immut);
	println!("> {:?}", immut);
	println!("> {:?}", b_nf);
}
