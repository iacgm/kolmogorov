use kolmogorov::*;

fn fib_ctx() -> (Context, Option<Analyzer>) {
	use Term::*;

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

	(context! { lte, plus, minus, one, two }, None)
}

fn fib(n: i32) -> i32 {
	if n <= 1 {
		n
	} else {
		fib(n - 1) + fib(n - 2)
	}
}

fn main() {
	use std::rc::Rc;
	use std::time::Instant;
	use Term::*;

	let targ = ty!((N => N) => N => N);

	let example = term!(f n -> lte n one one (plus (f (minus n one)) (f (minus n two))));
	println!("Example (|t| = {}): {}\n", example.size(), example);

	let (base_ctxt, _) = fib_ctx();
	let (mut exec_ctxt, analyzer) = fib_ctx();

	let limit = 8;
	let fibs: Rc<Vec<i32>> = Rc::new((0..limit).map(fib).collect());
	let prevs: Vec<(Identifier, BuiltIn)> = (1..limit)
		.map(|n| {
			let fibs2 = fibs.clone();
			let def = builtin! {
				N => N
				|c| => {
					let c = c.int()?;
					if 0 < c && c < n {
						Num(fibs2[c as usize])
					} else {
						Num(0)
					}
				}
			};
			let name: Identifier = format!("prevs_{}", n).leak();
			(name, def)
		})
		.collect();

	exec_ctxt.insert(&prevs[..]);

	let mut total_time = 0.;
	let mut search_time = 0.;
	let start = Instant::now();
	for size in 1.. {
		let now = Instant::now();
		println!("Total Time: {}", now.duration_since(start).as_secs_f32());
		println!("Searching Time: {}", search_time);
		println!("Execution Time: {}\n", total_time);
		println!("Searching size: {}", size);

		let mut search_start = Instant::now();
		'search: for (term, analysis) in search(base_ctxt.clone(), &targ, size, analyzer.clone()) {
			let search_end = Instant::now();

			search_time += search_end.duration_since(search_start).as_secs_f32();

			for n in 1..limit {
				let rec_arg = prevs[n as usize - 1].0;

				let program = term! {
					[term] [Var(rec_arg)] [Num(n)]
				};

				let start_exec = Instant::now();
				let output = exec_ctxt.evaluate(&program);
				let end_exec = Instant::now();

				total_time += end_exec.duration_since(start_exec).as_secs_f32();

				let output = output.int().unwrap();

				let expected = fib(n);
				if output != expected {
					search_start = Instant::now();
					continue 'search;
				}
			}

			println!("Term Found!");
			println!("{}", term);
			return;
		}
	}
}
