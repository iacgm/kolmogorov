use kolmogorov::*;

fn main() {
	use Term::*;

	let t = term!(x y -> x);
	let f = term!(x y -> y);

	let head = builtin! {
		forall a :: [a] => a
		using [t] in
		|_ctx, l| => term!([l] [t])
	};

	let tail = builtin! {
		forall a :: [a] => [a]
		using [f] in
		|_ctx, l| => term!([l] [f])
	};

	let cons = builtin! {
		forall a :: a => [a] => [a]
		|_ctx, h, t, f| => term!([f] [h] [t])
	};

	let sum = builtin! {
		:: N => N => N
		|ctx, a, b| => match (a.exec(ctx), b.exec(ctx)) {
			(Num(ref a), Num(ref b)) => Num(a+b),
			_ => unimplemented!()
		}
	};

	let length = builtin! {
		forall a :: [a] => N
		|ctx, l| => {
			match l.solve(ctx) {
				Var("nil") => Num(0),
				_ => term!(sum 1 (length (tail [l]))),
			}
		}
	};

	let mut context = Context::new(&[
		("sum", sum),
		("length", length),
		("head", head),
		("tail", tail),
		("cons", cons)
	]);

	let list = term!(cons 1 (cons 2 (cons 3 nil)));
	
	let mut len = term!(length [list]);

	println!("Length: {}", len);
	len.exec(&mut context);
	println!("Length: {}", len);
}
