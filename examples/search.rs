use kolmogorov::*;

fn main() {
	use Term::*;
	let plus = builtin!(
		N => N => N
		|x, y| => Num(x.int()? + y.int()?)
	);

	let mult = builtin!(
		N => N => N
		|x, y| => Num(x.int()? * y.int()?)
	);

	let zero = builtin!(
		N
		| | => Num(0)
	);

	let one = builtin!(
		N
		| | => Num(1)
	);

	let ctx = context! { plus, mult, zero, one };

	let ty = ty!(N => N);

	for n in 2.. {
		let start = std::time::Instant::now();

		let searcher = search::search(ctx.clone(), &ty, n);

		/* 		for term in searcher {
				   println!("FOUND: {}",term);
			   }
		*/
		let count: usize = searcher.count();

		println!(
			"There are {:>6} known-distinct programs of type {} and size {}.",
			count, ty, n
		);

		let end = std::time::Instant::now();

		println!("Time elapsed: {}s", end.duration_since(start).as_secs_f32());
	}
}
