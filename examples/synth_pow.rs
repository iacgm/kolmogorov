use kolmogorov::*;

fn main() {
	use Term::*;
	let ctxt = context();
	let targ = ty!(N => N => N);

	let example = term!(p n -> mult p (plus one one));
	println!("Example (|t| = {}): {}\n", example.size(), example);

	for size in 1.. {
		println!("Searching size {}:", size);
		'search: for term in search(ctxt.clone(), &targ, size) {
			for n in 1..5 {
				let prev = pow(n - 1);
				let expected = pow(n);

				let mut program = term! {
					[term] [Num(prev)] [Num(n)]
				};

				let mut env = Environment::new(ctxt.clone());
				env.execute(&mut program);

				let Term::Num(output) = program else {
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

fn pow(n: i32) -> i32 {
	2i32.pow(n as u32)
}

fn context() -> Context {
	use Term::*;

	let plus = builtin!(
		N => N => N
		|x, y| => Num(x.int()?+y.int()?)
	);

	let mult = builtin!(
		N => N => N
		|x, y| => Num(x.int()?*y.int()?)
	);

	let zero = builtin!(
		N
		| | => Num(0)
	);

	let one = builtin!(
		N
		| | => Num(1)
	);

	context! { plus, mult, zero, one }
}
