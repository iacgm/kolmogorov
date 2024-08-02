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

	let ty = ty!(N => N);

	for n in 2.. {
		let start = std::time::Instant::now();

		let searcher = Searcher::search(&dict, &ty, n);

		let mut count = 0;

		for term in searcher {
			count += 1;
			println!("{}", term);
		}

		println!(
			"These are all {:>6} known-distinct programs of type {} and size {}.",
			count, ty, n
		);

		let end = std::time::Instant::now();

		println!("Time elapsed: {}s", end.duration_since(start).as_secs_f32());
	}
}
