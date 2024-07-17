use kolmogorov::*;

fn main() {
	use Term::*;
	let succ = builtin!(
		N => N
		|x| => Num(1+x.int())
	);

	let plus = builtin!(
		N => N => N
		|x, y| => Num(x.int()+y.int())
	);

	let mult = builtin!(
		N => N => N
		|x, y| => Num(x.int()*y.int())
	);

	let zero = builtin!(
		N
		| | => Num(0)
	);

	let one = builtin!(
		N
		| | => Num(1)
	);

	let dict = dict! { plus, mult, zero, one, succ };

	println!("All small functions of size N:");

	for n in 1..10 {
		println!("N = {}", n);
		for term in enumerate(&dict, &ty!(N => N), n) {
			println!("|t|= {} where t = {}", term.size(), term);
		}
	}
}
