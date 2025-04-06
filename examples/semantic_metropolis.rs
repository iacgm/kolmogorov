use kolmogorov::*;

mod polynomials;
use polynomials::*;

fn main() {
	let lang = PolynomialLanguage;

	let num_examples = 10;

	let output = simple_map(
		lang,
		(0..num_examples).map(|n| (n, 4 * n * n * n + n * n)),
		term!(n -> n),
		ty!(N => N),
		Settings {
			bias: SizeBias::DistAbs { mean: 20, c: 0.5 },
			..Default::default()
		},
	);

	output.display(lang)
}
