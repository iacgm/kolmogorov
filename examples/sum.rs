use kolmogorov::*;

fn main() {
	use Term::*;

	let nil = term!(c n -> n);
	let cons = term!(h t c n -> c h (t c n));

	let plus = builtin!(
		N => N => N
		|x, y| => {
			let (Num(x), Num(y)) = (x, y) else {
				unimplemented!()
			};
			Num(x+y)
		}
	);

	let mut dictionary = Dictionary::new(&[("plus", plus)]);

	let list = term!([cons] 1 [nil]);

	let mut sum = term!([list] plus 0);

	dictionary.execute(&mut sum);

	println!("sum = {}", sum);
}
