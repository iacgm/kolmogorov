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
		| | => Num(1)
	);

	let dict = dict! { plus, mult, zero, one };

	//Interestingly, this is 4 * the catalan numbers
	for n in (3..).step_by(4) {
		let ty = ty!(N => N);
		let count: usize = enumerate(&dict, &ty, n).count();
		println!(
			"There are {:>6} programs of type {} and size {}.",
			count, ty, n
		);
	}
}
