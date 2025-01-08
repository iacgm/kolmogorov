use kolmogorov::*;

mod contexts;
use contexts::*;

fn main() {
	let lang = Polynomials;

	let ty = ty!(N => N);

	for n in 6..=6 {
		println!("Round {}", n);
		let start = std::time::Instant::now();

		let searcher = search(Box::new(lang.clone()), &ty, n);

		let mut count = 0;

		for (term, analysis) in searcher {
			count += 1;
			println!("\n{}", term);
			if let Analysis::Canonical(sem) = analysis {
				println!("≈ {}", sem);
			}
		}
		println!();

		println!(
			"These are all {:>6} known-distinct programs of type {} and size {}.",
			count, ty, n
		);

		let end = std::time::Instant::now();

		println!("Time elapsed: {}s", end.duration_since(start).as_secs_f32());

		std::thread::sleep(std::time::Duration::from_secs(1));
	}
}
