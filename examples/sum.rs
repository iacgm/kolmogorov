use kolmogorov::*;

fn main() {
	use Term::*;

	let nil = term!(c n -> n);
	let cons = term!(h t c n -> c h (t c n));

	let plus = builtin!(
		N => N => N
		|x, y| => Num(x.int()?+y.int()?)
	);

	let ctxt = context! { plus };

	let list = term!([cons] 1 [nil]);

	let sum = term!([list] plus 0);

	let output = ctxt.evaluate(&sum);

	println!("sum = {}", output);
}
