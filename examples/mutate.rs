use kolmogorov::*;

mod opaque;
use opaque::*;

fn main() {
	let lang = Opaque;

	let inc = term!(plus one one);

	let mutated = random::mutate(Box::new(lang), &inc);

	println!("Old: {}", inc);
	println!("New: {}", mutated);
}
