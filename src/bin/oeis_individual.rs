mod languages;
mod utils;

use kolmogorov::*;
use languages::*;
use utils::*;

fn main() -> std::io::Result<()> {
    let lang = Polynomials;
    let oeis = oeis::load_oeis_def()?;

    let mut output_file = std::fs::File::create("data/oeis_individual")?;

    println!("{} sequences:", oeis.seq.len());

    let mut keys = oeis.seq.keys().cloned().collect::<Vec<_>>();
    keys.sort();
    keys = vec![384];

    for id in keys {
        let nums = &oeis.seq[&id];

        let examples =
            nums.iter().cloned().enumerate().map(|(i, n)| (i as i32, n));

        let output = simple_map(
            lang,
            examples,
            None,
            ty!(N => N),
            SynthesisParameters {
                bias: SizeBias::DistAbs { mean: 25, c: 0.5 },
                iterations: 100_000,
                ..Default::default()
            },
            Options { print_freq: None },
        );

        output.display(lang);
    }

    Ok(())
}
