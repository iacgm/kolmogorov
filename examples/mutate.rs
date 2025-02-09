use kolmogorov::*;

mod opaque;
use opaque::*;

fn main() {
	let lang = Opaque;

	let inc = term!(plus one);

	let ty = ty!(N => N);

	let mutated = random::mutate(Box::new(lang), &inc, &ty);

	println!("Old: {}", inc);
	println!("New: {}", mutated);
}
