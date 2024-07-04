use kolmogorov::*;

fn main() {
	let base = ty!(forall a b c :: (a => b) => (b => c) => (a => c));
	let inst = &ty!((N => N) => (N => N) => (N => N)).mono;
	let diff = &ty!((b => c) => (a => b) => (a => c)).mono; 

	println!("{} <:? {}: {}", inst, base, base.instantiates(inst));
	println!("{} <:? {}: {}", diff, base, base.instantiates(diff));

	println!("{} unify {}: {:?}", base, inst, diff.unify(inst));
	println!("{} unify {}: {:?}", base, inst, diff.unify(inst));
}
