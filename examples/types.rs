use kolmogorov::*;

fn main() {
	let base = poly!(forall a b c :: (a => b) => (b => c) => (a => c));
	let inst = mono!((N => y) => (x => N) => (x => y));
	let diff = mono!((b => c) => (a => b) => (a => c));

	println!("{} <:? {}: {}", inst, base, base.instantiates(&inst));
	println!("{} <:? {}: {}", diff, base, base.instantiates(&diff));

	let mut sub = TypeSub::default();
	dbg!(sub.unify(&diff, &inst));
	
	let mut unified = diff.clone();
	sub.to_mono(&mut unified);

	println!("{} unify {}: \n\t{:?}\n\tyielding:\n\t{}", base, inst, sub, unified);

	judge(&term!(h t c n -> c h (t c n)));
	judge(&term!(f g x -> f (g x)));
	judge(&term!(f x -> f (f x)));
	judge(&term!(a b -> a));
	judge(&term!(f x -> f x));
	judge(&term!(x -> x));
}

fn judge(term: &Term) {
	let context = Dictionary::new(&[]);
	match context.infer(term) {
		Some(ty) => println!("{} : {}", term, ty),
		None => println!("{} is untypable.", term),
	}
}
