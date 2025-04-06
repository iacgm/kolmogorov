use super::*;

use statrs::distribution::{Continuous, Normal};

pub struct SynthesisParameters {
	pub bias: SizeBias,
	pub score_factor: f64,
	pub iterations: usize,
}

pub struct MetropolisOutput {
	pub term: Term,
	pub iterations: usize,
	pub time: f64,
	pub num_correct: usize,
	pub score: Option<f64>,
}

impl Default for SynthesisParameters {
	fn default() -> Self {
		Self {
			bias: SizeBias::Unbiased,
			score_factor: 0.5,
			iterations: 50_000,
		}
	}
}

pub fn simple_map<L, I, O>(
	lang: L,
	examples: impl Iterator<Item = (I, O)>,
	start: Term,
	ty: Type,
	settings: SynthesisParameters,
	options: Options,
) -> MetropolisOutput
where
	L: Language,
	I: TermValue + Clone,
	O: TermValue + Clone,
{
	let examples = examples.map(|(i, o)| (Term::val(i), o)).collect::<Vec<_>>();
	
	let num_examples = examples.len();

	let lang_ctxt = lang.context();

	let int_scorer = |t: &Term| {
		let mut num_correct = 0;
		for (i, o) in examples.iter() {
			let program = term! {
				[t] [i]
			};

			let evaled = lang_ctxt.evaluate(&program);

			let Some(output) = evaled.leaf_val() else {
				unimplemented!("Term `{}` did not evaluate to value.", evaled);
			};

			if o.is_eq(&output) {
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

		let prob_score = (settings.score_factor * num_correct as f64).exp();
		Some(settings.bias.apply(prob_score, term.size()))
	};

	let start_time = std::time::Instant::now();
	let (iterations, term) = metropolis(&lang, &start, &ty, scorer, settings.iterations, options);
	let end_time = std::time::Instant::now();

	let num_correct = int_scorer(&term);
	let score = scorer(&term);

	MetropolisOutput {
		term,
		iterations,
		time: end_time.duration_since(start_time).as_secs_f64(),
		num_correct,
		score,
	}
}

// Used to bias programs towards reasonable sizes / prevent runaway term sizes
#[derive(Clone, Copy)]
pub enum SizeBias {
	Unbiased,
	LinearBeyond { cutoff: usize, c: f64 },
	Norm { m: f64, s: f64 },
	DistAbs { mean: usize, c: f64 },
}

impl SizeBias {
	pub fn apply(self, score: f64, size: usize) -> f64 {
		use SizeBias::*;
		match self {
			Unbiased => score,
			LinearBeyond { cutoff, c } => {
				let punishment = -c * size.saturating_sub(cutoff) as f64;

				score * punishment.exp()
			}
			Norm { m, s } => {
				let normal = Normal::new(m, s).unwrap();

				score * normal.pdf(size as f64)
			}
			DistAbs { mean, c } => {
				let dist = if mean >= size {
					mean - size
				} else {
					size - mean
				};

				let punishment = -c * dist as f64;

				score * punishment.exp()
			}
		}
	}
}

impl MetropolisOutput {
	pub fn display<L: Language>(&self, lang: L) {
		let MetropolisOutput {
			term,
			iterations,
			time,
			num_correct,
			score,
		} = self;

		println!("Best Found: {}", &term);
		println!("Semantics:  {}", lang.analyze(term));

		println!("Score: {:?} (or {:?} correct)", score, num_correct,);

		println!("Iterations: {}", iterations);
		println!("Time (s): {}", time);
		println!("Time (s/iter): {}", time / *iterations as f64);
	}
}
