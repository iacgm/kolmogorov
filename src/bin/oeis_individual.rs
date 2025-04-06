mod languages;
mod utils;

use kolmogorov::*;
use languages::*;
use utils::*;

fn main() -> std::io::Result<()> {
	let lang = Polynomials;
	let oeis = oeis::load_oeis()?;

	let mut output_file = std::fs::File::create("data/oeis_individual")?;

	println!("{} sequences:", oeis.len());

	let mut keys = oeis.keys().collect::<Vec<_>>();
	keys.sort();

	for id in keys {
		let nums = &oeis[id];

		let examples = nums.iter().cloned().enumerate().map(|(i, n)| (i as i32, n));

		let output = simple_map(
			lang,
			examples,
			term!(n -> n),
			ty!(N => N),
			SynthesisParameters {
				bias: SizeBias::DistAbs { mean: 20, c: 0.5 },
				..Default::default()
			},
			Options { print_freq: None },
		);

		if output.score.is_none() {
			use std::io::*;

			let term = output.term;
			let analysis = lang.analyze(&term);

			let text = format!("Solution found for A{:06}: {} (≈ {})", id, term, analysis);

			println!("{}", text);
			write!(output_file, "{}", text)?;
			output_file.flush()?;
		}
	}

	Ok(())
}
