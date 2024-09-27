use kolmogorov::*;

mod contexts;
use contexts::*;

fn main() {
	let lang = Polynomials;
	let ty = ty!(N => N);

	for n in 2.. {
		let start = std::time::Instant::now();

		let searcher = search::search(Box::new(lang.clone()), &ty, n);

		let count: usize = searcher.count();

		println!(
			"There are {:>6} known-distinct programs of type {} and size {}.",
			count, ty, n
		);

		let end = std::time::Instant::now();

		println!("Time elapsed: {}s", end.duration_since(start).as_secs_f32());
	}
}
