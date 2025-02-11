use kolmogorov::{random::metropolis, *};

mod opaque;
use opaque::*;

fn main() {
	let lang = Opaque;

	let examples: Vec<_> = (0..10).map(|n| (n, n * n)).collect();

	let lang_ctxt = lang.context();

	const TUNING_PARAM: f64 = 0.5;

	let scorer = |t: &Term| {
		use Term::*;
		let mut num_correct = 0;
		for (x, y) in examples.iter().copied() {
			let program = term! {
				[t] [Num(x)]
			};

			println!("> {}", &program);

			let output = lang_ctxt.evaluate(&program).int().unwrap();

			if output == y {
				num_correct += 1;
			}
		}

		(TUNING_PARAM * num_correct as f64).exp()
	};

	let start = term!(n -> n);

	let ty = ty!(N => N);

	let iterations = 1000;

	let metropolis_search = metropolis(&lang, &start, &ty, scorer, iterations);

	println!("Best Found: {}", metropolis_search);
}
