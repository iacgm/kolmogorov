use kolmogorov::*;

mod contexts;
use contexts::*;

use std::time::Instant;

fn pow(n: i32) -> i32 {
	2i32.pow(n as u32)
}

fn main() {
	use Term::*;
	let ctxt = polynomials();
	let targ = ty!(N => N => N);

	let example = term!(p n -> mult p (plus one one));
	println!("Example (|t| = {}): {}\n", example.size(), example);

	let mut total_time = 0f32;

	for size in 1.. {
		println!("Time: {}", total_time);
		println!("Searching size {}:", size);
		'search: for term in search(ctxt.clone(), &targ, size) {
			for n in 1..5 {
				let prev = pow(n - 1);
				let expected = pow(n);

				let mut program = term! {
					[term] [Num(prev)] [Num(n)]
				};

				
				let start = Instant::now();
				let mut env = Environment::new(ctxt.clone());
				env.execute(&mut program);
				let end = Instant::now();

				let output = program;

				total_time += end.duration_since(start).as_secs_f32();

				let Term::Num(output) = output else {
					unreachable!()
				};

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
