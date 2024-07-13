use kolmogorov::*;

fn main() {
	use Term::*;
	let plus = builtin!(
		N => N => N
		|x, y| => Num(x.int()+y.int())

	);

	let mult = builtin!(
		N => N => N
		|x, y| => Num(x.int()*y.int())
	);

	let zero = builtin!(
		N
		| | => Num(0)
	);

	let one = builtin!(
		N
		| | => Num(0)
	);

	let dictionary = dict! { plus, mult, zero, one };

	let ty = ty!(N);

	let generated = generate_term(&dictionary, &ty, 10);

	println!("Generating term of type {}.", ty);
	println!("Generated {}.", generated.unwrap_or(Var("-")));

/* 		let inferred = dictionary.infer(&generated).unwrap();
		println!("Inferred type: {}.", &inferred); */
}
