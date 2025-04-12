use kolmogorov::*;

mod languages;
use languages::*;

fn main() {
	let lang = Polynomials;

	let num_examples = 10;

	let output = iterative(
		lang,
		1i32,
		(1..num_examples).map(|n| (n as i32, 2i32.pow(n))),
		term!(n i -> n),
		ty!(N => N => N),
		SynthesisParameters {
			bias: SizeBias::DistAbs { mean: 10, c: 0.5 },
			..Default::default()
		},
		Options::default(),
	);

	output.display(lang)
}
