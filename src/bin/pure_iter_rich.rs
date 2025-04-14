mod languages;
mod utils;

use kolmogorov::*;
use languages::*;
use utils::*;

fn main() -> std::io::Result<()> {
    let lang = CondPolyLang;
    let oeis = oeis::load_oeis_def()?;

    let mut output_file = std::fs::File::create("data/pure_iter_rich")?;

    println!("{} sequences:", oeis.seq.len());

    let mut keys = oeis.seq.keys().collect::<Vec<_>>();
    keys.sort();

    for id in keys {
        let nums = &oeis.seq[id];

        let examples = nums[1..].iter().cloned();

        let output = pure_iterative(
            lang,
            nums[0],
            examples,
            None,
            ty!(Poly => N),
            SynthesisParameters {
                bias: SizeBias::DistAbs { mean: 20, c: 0.5 },
                iterations: 75_000,
                ..Default::default()
            },
            Options { print_freq: None },
        );

        if output.score.is_none() {
            use std::io::*;

            let term = output.term;
            let analysis = lang.analyze(&term);

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
