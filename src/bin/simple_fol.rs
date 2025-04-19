mod languages;
mod utils;

use kolmogorov::*;
use languages::*;
use utils::*;

fn main() -> std::io::Result<()> {
    let lang = LogicLang::new(1);

    let opts = OEISLoadOptions {
        required: vec!["nonn"],
        max_val: 88,
        ..Default::default()
    };
    let oeis = load_oeis(&opts)?;

    let mut output_file = std::fs::File::create("data/oeis_individual")?;

    println!("{} sequences:", oeis.seq.len());

    let mut keys = oeis.seq.keys().collect::<Vec<_>>();
    keys.sort();

    let keys = vec![18252];

    for id in keys {
        let nums = &oeis.seq[&id]
            .iter()
            .map(|n| *n as u32)
            .collect::<Vec<u32>>();

        let examples =
            (2u32..opts.max_val as u32).map(|n| (n, nums.contains(&n)));

        let output = simple_map(
            lang.clone(),
            examples,
            None,
            ty!(N => Bool),
            SynthesisParameters {
                bias: SizeBias::DistAbs { mean: 20, c: 0.5 },
                ..Default::default()
            },
            Options {
                print_freq: Some(1),
            },
        );

        if output.score.is_none() {
            use std::io::*;

            let term = output.term;
            let semantics = output.analysis.canon();

            let text = format!(
                "Solution found for A{:06}: {} (≈ {})",
                id, term, semantics
            );

            println!("{}", text);
            writeln!(output_file, "{}", text)?;
            output_file.flush()?;
        }
    }

    Ok(())
}
