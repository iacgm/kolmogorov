use kolmogorov::*;

fn main() {
	use Term::*;

	let fix = literal!{
		forall a :: (a => a) => a
		|f| => term!([f] (fix [f]))
	};

	let cons = term!(h t f -> f h t);

	let ones = term!(fix ([cons] 1));

	let empty = literal!{
		forall a b :: [a] => (b => b => b)
		|l| => {
			match l {
				Var("nil") => term!(a b -> a),
				_ => term!(a b -> b),
			}
		}
	};

	let mut test = term!(empty [ones]);
	
	let mut context = Context::new(&[
		("fix", fix),
		("non_empty", empty),
	]);

	test.exec(&mut context);

	println!("Final result: {}", test);

}
