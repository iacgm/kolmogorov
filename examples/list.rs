use kolmogorov::*;

fn main() {
	use Term::*;
	let cons = term!("h" "t" "s" -> "s" "h" "t");
	let t = term!("x" "y" -> "x");
	let f = term!("x" "y" -> "y");
	let nil = term!("nil");

	let isnil = literal!(isnil:
		[!t] => {
			match t {
				Var("nil") => term!("x" "y" -> "x"),
				_ => term!("x" "y" -> "y"),
			}
		};
	);

	let head = term!("l" -> "l" t);
	let tail = term!("l" -> "l" f);

	let make_list = |elems: &[i32]| {
		let mut out = term!(nil);
		for elem in elems.iter().rev() {
			let elem = Term::Num(*elem);
			out = term!(cons[elem][out]);
		}
		out
	};

	let m = term!("x" -> "x" "x");
	let y = term!("f" -> m ("x" -> ("f" ("x" "x"))));

	let fold_f = literal!(fold:
		[r, n, f, !l] =>
			match l {
				Var("nil") => n,
				_ => term!(f (l ("x" "y" -> "x")) (r n f (l ("x" "y" -> "y"))))
			};
	);

	let fold = term!(y fold_f);

	let sum = literal!(sum:
		[!x, !y] => {
			match (x, y) {
				(Num(x), Num(y)) => Num(x+y),
				_ => unimplemented!()
			}
		};
	);

	let sum_up = term!(fold 0 sum);

	let length = term!(fold 0 ("x" "y" -> sum 1 "y"));

	let list = make_list(&[1, 2, 3, 4, 5]);

	exec(&term!(sum_up list));
	exec(&term!(length list));
}

fn exec(term: &Term) {
	let mut term = term.clone();
	println!("{}:", term);
	term.normalize();
	println!("\t{}", term);
}
