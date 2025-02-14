use kolmogorov::*;

mod opaque;
use opaque::*;

fn main() {
	let lang = Opaque;

	let inc = term!(n -> n);

	let ty = ty!(N => N);

	let (mutated, _) = random::mutate(&lang, &inc, &ty).unwrap();

	println!("Old: {}", inc);
	println!("New: {}", mutated);
}
