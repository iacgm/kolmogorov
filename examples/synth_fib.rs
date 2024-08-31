use kolmogorov::*;

mod contexts;
use contexts::*;

fn fib(n: i32) -> i32 {
	if n <= 1 {
		n
	} else {
		fib(n - 1) + fib(n - 2)
	}
}

fn main() {
	use std::time::Instant;
	use Term::*;

	let ctxt = fib_ctx();
	let targ = ty!((N => N) => N => N);

	let example = term!(f n -> lte n one one (plus (f (minus n one)) (f (minus n two))));
	println!("Example (|t| = {}): {}\n", example.size(), example);

	let start = Instant::now();

	for size in 1.. {
		println!(
			"Time: {}",
			Instant::now().duration_since(start).as_secs_f32()
		);
		println!("Searching size {}:", size);
		'search: for term in search(ctxt.clone(), &targ, size) {
			for n in 1..8 {
				let mut ctxt = ctxt.clone();

				let prevs = builtin!(
					N => N
					|c| => {
						let c = c.int()?;
						if c < n {
							Num(fib(c))
						} else {
							Num(0)
						}
					}
				);

				ctxt.insert(&[("prevs", prevs)]);

				let program = term! {
					[term] prevs [Num(n)]
				};

				let output: Term = ctxt.evaluate(program).into();

				let Term::Num(output) = output else {
					unreachable!()
				};

				let expected = fib(n);
				if output != expected {
					continue 'search;
				}
			}

			println!("Term Found!");
			println!("{}", term);
			return;
		}
	}
}
