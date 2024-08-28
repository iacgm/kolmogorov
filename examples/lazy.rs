use kolmogorov::*;

fn main() {
	let ctxt = context! {};

	let program = term!{
		(a b -> a) ((x -> x) o) ((y -> y) t)
	};

	dbg!(ctxt.evaluate(program));

}
