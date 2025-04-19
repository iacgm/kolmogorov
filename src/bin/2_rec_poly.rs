mod languages;
mod utils;

use kolmogorov::*;
use languages::*;
use utils::*;

fn main() -> std::io::Result<()> {
    let lang = Polynomials;
    let oeis = oeis::load_oeis_def()?;

    let mut output_file = std::fs::File::create("data/2_rec_poly")?;

    println!("{} sequences:", oeis.seq.len());

    let mut keys = oeis.seq.keys().collect::<Vec<_>>();
    keys.sort();

    for id in keys {
        let nums = &oeis.seq[id];

        let output = k_rec::<Polynomials, i32>(
            lang,
            2,
            nums.iter().cloned(),
            term!(a b -> plus a b),
            ty!(N => N => N),
            SynthesisParameters {
                bias: SizeBias::DistAbs { mean: 20, c: 0.5 },
                iterations: 50_000,
                ..Default::default()
            },
            Options { print_freq: None },
        );

        if output.score.is_none() {
            use std::io::*;

            let MetropolisOutput { term, analysis, .. } = output;

            let text = format!(
                "Solution found for A{:06}: {} (≈ {})",
                id, term, analysis
            );

            println!("{}", text);
            writeln!(output_file, "{}", text)?;
            output_file.flush()?;
        }
    }

    Ok(())
}
