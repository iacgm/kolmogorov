use kolmogorov::*;

fn main() {
	use Term::*;

	let tr = term!(a b -> a);
	let fl = term!(a b -> a);

	let fix = builtin! {
		forall a :: (a => a) => a
		|f| => term!([f] (fix [f]))
	};

	let nil = term!(c n -> n);

	let cons = builtin! {
		forall a :: a => [a] => [a]
		|h, t| => term!(c n -> c [h] ([t] c n))
	};

	let head = builtin! {
		forall a :: [a]
		|l| => term!([l] (h t -> h) (a b -> b))
	};

	let tail = builtin! {
		forall a :: [a] => [a]
		|l| => term!(c n -> [l] (h t g -> g h (t c)) (t -> n) (h t -> t))
	};

	let isnil = builtin! {
		forall a :: [a] => Bool
		using [tr, fl] in
		|l| => term!([l] (h t -> [fl]) [tr])
	};

	let pred = builtin! {
		:: N => N
		|n| => match n {
			Num(n) => Num(n-1),
			_ => unimplemented!(),
		}
	};

	let take = builtin! {
		:: N => Bool
		using [nil] in
		|n| => match n {
			Num(0) => term!(l -> [nil]),
			_ => term!(l -> cons (head l) (take (pred [n]) (tail l)))
		}
	};

	let mul = builtin! {
		:: N => N
		|a, b| => match (a, b) {
			(Num(a), Num(b)) => Num(a*b),
			_ => unimplemented!()
		}
	};

	let add = builtin! {
		:: N => N
		|a, b| => match (a, b) {
			(Num(a), Num(b)) => Num(a+b),
			_ => unimplemented!()
		}
	};

	let fact = builtin! {
		:: N => N
		|n| => match n {
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

	let ints = term!(fix (cons 1));//make_list(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

	let mut list = term!([sum_up] (take 5 [ints]));

	let mut context = Context::new(&[
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
