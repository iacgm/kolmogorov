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

	let ctx = context! { plus, mult, zero, one };

	let ty = ty!(N => N);

	const N : usize = 22;

	let start = std::time::Instant::now();

	let searcher = Searcher::search(ctx.clone(), &ty, N);

	let count: usize = searcher.count();

	println!(
		"There are {:>6} known-distinct programs of type {} and size {}.",
		count, ty, N
	);

	let end = std::time::Instant::now();

	println!("Time elapsed: {}s", end.duration_since(start).as_secs_f32());

}
