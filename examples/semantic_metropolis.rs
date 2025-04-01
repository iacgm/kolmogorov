use kolmogorov::{random::metropolis, *};

mod polynomials;
use polynomials::*;

fn main() {

	let lang = PolynomialLanguage;

	let num_examples = 10;

	let examples: Vec<_> = (0..num_examples)
		.map(|n| (n, 4 * n * n * n + n * n))
		.collect();

	let lang_ctxt = lang.context();

	const TUNING_PARAM: f64 = 0.5;

	let int_scorer = |t: &Term| {
		use Term::*;
		let mut num_correct = 0;
		for (x, y) in examples.iter().copied() {
			let program = term! {
				[t] [Num(x)]
			};

			let evaled = lang_ctxt.evaluate(&program);

			let output = evaled.int().unwrap();

			if output == y {
				num_correct += 1;
			}
		}

		num_correct
	};

	let scorer = |term: &Term| {
		let num_correct = int_scorer(term);

		if num_examples == num_correct {
			return None;
		}

		Some((TUNING_PARAM * num_correct as f64).exp())
	};

	let start = term!(n -> n);

	let ty = ty!(N => N);

	let iterations = 50_000;

	let metropolis_search = metropolis(&lang, &start, &ty, scorer, iterations);

	println!("Best Found: {}", &metropolis_search);
	println!("Semantics:  {}", lang.analyze(&metropolis_search));

	println!(
		"Score: {:?} (or {:?} correct)",
		scorer(&metropolis_search),
		int_scorer(&metropolis_search),
	);
}
