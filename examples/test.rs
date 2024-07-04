use kolmogorov::*;

fn main() {
	let nil = term!(c n -> n);
	let cons = term!(h t c n -> c h (t c n));

	let mut prog = term!([cons] 2 ([cons] 1 [nil]) f n);

	println!(">{}", prog);
	prog.normalize();
	println!(">{}", prog);
}
