use kolmogorov::*;

fn main() {
	use Term::*;

	let t = term!(x y -> x);
	let f = term!(x y -> y);

	let head = literal!{
		((a => b => a) => a) => a
		[l] => term!([l] [t])
	};

	let tail = literal!{
		((a => b => b) => b) => b
		[l] => term!([l] [f])
	};

	let cons = literal!{
		N => a => a
		[h, t, f] => term!([f] [h] [t])
	};

	let sum = literal!{
		N => N => N
		[x, y] => {
			match (x, y) {
				(Num(ref x), Num(ref y)) => Num(x+y),
				_ => unimplemented!(),
			}
		}
	};

	let length = literal!{
		a => N
		[l] => {
			match l {
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
