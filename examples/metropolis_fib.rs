use statrs::distribution::{Continuous, Normal};
use std::rc::Rc;

use kolmogorov::{random::metropolis, *};

mod fib_lang;
use fib_lang::*;

fn main() {
	let lang = FibLang;

	let num_examples = 10;

	let examples: Vec<_> = (0..num_examples).map(|n| (n, fib(n))).collect();

	let lang_ctxt = lang.context();

	let mut exec_ctxt = lang_ctxt.clone();

	let fibs: Rc<Vec<i32>> = Rc::new((0..num_examples).map(fib).collect());
	let prevs: Vec<(Identifier, BuiltIn)> = (0..num_examples)
		.map(|n| {
			use Term::*;
			let fibs2 = fibs.clone();
			let def = builtin! {
				N => N
				|c| => {
					let c = c.int()?;
					if 0 < c && c < n {
						Val(fibs2[c as usize])
					} else {
						Val(0)
					}
				}
			};
			let name: Identifier = Identifier::Name(format!("prevs_{}", n).leak());
			(name, def)
		})
		.collect();

	exec_ctxt.insert(&prevs);

	const SCORE_TUNING_PARAM: f64 = 0.5;
	const SIZE_TUNING_PARAM: f64 = 1.0;

	let avg_size = 15f64;
	let size_std = 5f64;

	let normal = Normal::new(avg_size, size_std).unwrap();

	let size_scorer = |s: usize| lerp(SIZE_TUNING_PARAM, 1., normal.pdf(s as f64));

	let int_scorer = |t: &Term| {
		use Term::*;
		let mut num_correct = 0;
		for (n, f_n) in examples.iter().copied() {
			let rec_arg = prevs[n as usize].0;

			let program = term! {
				[t] [Var(rec_arg)] [Val(n)]
			};

			let evaled = exec_ctxt.evaluate(&program);

			let output = evaled.int().unwrap();

			if output == f_n {
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

		let prob_score = (SCORE_TUNING_PARAM * num_correct as f64).exp();
		let prob_size = size_scorer(term.size());

		Some(prob_score * prob_size)
	};

	let start = term!(f n -> plus (f (minus n one)) (f (minus n two)));

	let ty = ty!((N => N) => N => N);

	let iterations = 150_000;

	let metropolis_search = metropolis(&lang, &start, &ty, scorer, iterations);

	println!("Best Found: {}", &metropolis_search);
	println!("Semantics:  {}", lang.analyze(&metropolis_search));

	println!(
		"Score: {:?} (or {:?} correct)",
		scorer(&metropolis_search),
		int_scorer(&metropolis_search),
	);
}

fn lerp(t: f64, a: f64, b: f64) -> f64 {
	(1. - t) * a + t * b
}

fn fib(n: i32) -> i32 {
	if n <= 1 {
		n
	} else {
		fib(n - 1) + fib(n - 2)
	}
}
