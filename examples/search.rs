use kolmogorov::*;

mod contexts;
use contexts::*;

fn main() {
	let ctx = polynomials();
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
