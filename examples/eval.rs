use kolmogorov::*;

mod fib_lang;
use fib_lang::*;

fn main() {
	let lang = FibLang;

	let term = term!(lte two one x y);

	let output = lang.context().evaluate(&term);

	println!("{} \n -> {}", term, output);
}
