use kolmogorov::*;

mod opaque;
use opaque::*;

fn main() {
	let lang = Opaque;
	let ctxt = lang.context();

	let term = term!(b -> plus 1 b);

	let inferred = ctxt.infer_type(&term);

	println!("Term:     {}", term);
	println!("Has Type: {}", inferred);
}
