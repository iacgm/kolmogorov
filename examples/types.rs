use kolmogorov::*;

#[allow(non_snake_case)]
fn main() {
	let base = ty!((a => b) => (b => c) => (a => c));
	let inst = ty!((N => y) => (x => N) => (x => y));
	let diff = ty!((b => c) => (a => b) => (a => c));

	let mut sub = TypeSub::default();
	dbg!(sub.unify(&mut diff.clone(), &mut inst.clone()));

	let mut unified = diff.clone();
	sub.apply(&mut unified);

	println!(
		"{} unify {}: \n\t{:?}\n\tyielding:\n\t{}",
		base, inst, sub, unified
	);

	judge(&term!(h t c n -> c h (t c n)));
	judge(&term!(f g x -> f (g x)));
	judge(&term!(f x -> f (f x)));
	judge(&term!(a b -> a));
	judge(&term!(f x -> f x));
	judge(&term!(x -> x));

	let K = term!(x y -> x);
	judge(&term!(f x -> [K] (f x) (f 1)));
}

fn judge(term: &Term) {
	let context = dict! {};
	match context.infer(term) {
		Some(ty) => println!("{} : {}", term, ty),
		None => println!("{} is untypable.", term),
	}
}
