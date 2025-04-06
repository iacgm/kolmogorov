mod languages;
mod utils;

use kolmogorov::*;
use languages::*;
use utils::*;

fn main() -> std::io::Result<()> {
	let lang = Polynomials;
	let oeis = oeis::OEISMap::load()?;

	for (id, nums) in oeis.iter() {
		println!("Searching for: A{:06}", id);

		let examples = nums.iter().cloned().enumerate().map(|(i, n)| (i as i32, n));

		let output = simple_map(
			lang,
			examples,
			term!(n -> n),
			ty!(N => N),
			SynthesisParameters {
				bias: SizeBias::DistAbs { mean: 20, c: 0.5 },
				iterations: 100_000,
				..Default::default()
			},
			Options { print_freq: None },
		);

		if output.score.is_none() {
			let term = output.term;
			let analysis = lang.analyze(&term);
			println!("Solution found for {}: {} (≈ {})", id, term, analysis);
		}
	}

	Ok(())
}
