use kolmogorov::*;

fn main() {
	use Term::*;

	let tr = term!(a b -> a);
	let fl = term!(a b -> a);

	let fix = builtin! {
		forall a :: (a => a) => a
		|_ctx, f| => term!([f] (fix [f]))
	};

	let nil = term!(c n -> n);

	let cons = builtin! {
		forall a :: a => [a] => [a]
		|_ctx, h, t| => term!(c n -> c [h] ([t] c n))
	};

	let head = builtin! {
		forall a :: [a]
		|_ctx, l| => term!([l] (h t -> h) (a b -> b))
	};

	let tail = builtin! {
		forall a :: [a] => [a]
		|_ctx, l| => term!(c n -> [l] (h t g -> g h (t c)) (t -> n) (h t -> t))
	};

	let isnil = builtin! {
		forall a :: [a] => Bool
		using [tr, fl] in
		|_ctx, l| => term!([l] (h t -> [fl]) [tr])
	};

	let pred = builtin! {
		:: N => N
		|ctx, n| => match *n.exec(ctx) {
			Num(n) => Num(n-1),
			_ => unimplemented!(),
		}
	};

	let take = builtin! {
		forall a :: N => [a] => [a]
		using [nil] in
		|ctx, n, l| => match n.exec(ctx) {
			Num(0) => nil.clone(),
			_ => term!(cons (head [l]) (take (pred [n]) (tail [l])))
		}
	};

	let mul = builtin! {
		:: N => N => N
		|ctx, a, b| => match (a.exec(ctx), b.exec(ctx)) {
			(Num(ref a), Num(ref b)) => Num(a*b),
			_ => unimplemented!()
		}
	};

	let add = builtin! {
		:: N => N => N
		|ctx, a, b| => match (a.exec(ctx), b.exec(ctx)) {
			(Num(ref a), Num(ref b)) => Num(a+b),
			_ => unimplemented!()
		}
	};

	let fact = builtin! {
		:: N => N
		|ctx, n| => match n.exec(ctx) {
			Num(0) => Num(1),
			_ => term!(mul [n] (fact (pred [n]))),
		}
	};

	let sum_up = term!(l -> l add 0);

	let make_list = |list: &[i32]| {
		let mut l = nil.clone();
		for &n in list.iter().rev() {
			l = term!(cons[Num(n)][l])
		}
		l
	};

	let ints = make_list(&[0,1,2,3,4,5,6,7,8,9]);

	let mut list = term!([sum_up] (take 5 [ints]));

	let mut context = Dictionary::new(&[
		("isnil", isnil),
		("tail", tail),
		("head", head),
		("cons", cons),
		("fix", fix),
		("take", take),
		("pred", pred),
		("fact", fact),
		("mul", mul),
		("add", add),
	]);

	println!("{}", list);
	list.exec(&mut context);
	println!(">{}", list);
}
