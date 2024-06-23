use kolmogorov::*;

fn main() {
	use Term::*;

	let t = term!(x y -> x);
	let f = term!(x y -> y);

	let head = literal!{[l] => term!([l] [t])};
	let tail = literal!{[l] => term!([l] [f])};

	let nil = term!(nil);

	let cons = literal!{[h, t, f] => term!([f] [h] [t])};

	let sum = literal!{
		[x, y] => {
			
			match (x, y) {
				(Num(ref x), Num(ref y)) => Num(x+y),
				_ => unimplemented!(),
			}
		}
	};

	let length = literal!{
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

	let mut list = term!(cons 1 (cons 2 (cons 3 nil)));
	
	let mut len = term!(length [list]);

	println!("Length: {}", len);
	len.exec(&mut context);
	println!("Length: {}", len);
}
