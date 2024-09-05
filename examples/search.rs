use kolmogorov::*;

pub fn ctxt() -> Context {
	use NTerm::*;

	let lte = builtin!(
		N => N => N => N => N
		|a, b| => if a.int()? <= b.int()? {
			term!(a b -> a)
		} else {
			term!(a b -> b)
		}
	);

	let plus = builtin!(
		N => N => N
		|x, y| => Num(x.int()?+y.int()?)
	);

	let minus = builtin!(
		N => N => N
		|x, y| => Num(x.int()?-y.int()?)
	);

	let one = builtin!(
		N
		| | => Num(1)
	);

	let two = builtin!(
		N
		| | => Num(2)
	);

	context! { lte, plus, minus, one, two }
}

fn main() {
	let ctx = ctxt();
	let ty = ty!((N => N) => N => N);

	for n in 2.. {
		let start = std::time::Instant::now();

		let searcher = search::search(ctx.clone(), &ty, n);

		let count: usize = searcher.count();

		println!(
			"There are {:>6} known-distinct programs of type {} and size {}.",
			count, ty, n
		);

		let end = std::time::Instant::now();

		println!("Time elapsed: {}s", end.duration_since(start).as_secs_f32());
	}
}
