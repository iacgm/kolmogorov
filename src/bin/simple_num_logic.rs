mod languages;
mod utils;

use kolmogorov::*;
use languages::*;
use utils::*;

fn main() -> std::io::Result<()> {
    let lang = NumLogic::new(2);

    let opts = OEISLoadOptions {
        required: vec!["nonn"],
        disallow: vec![],
        max_val: 227,
        ..Default::default()
    };
    let oeis = load_oeis(&opts)?;

    let mut output_file = std::fs::File::create("data/oeis_individual")?;

    println!("{} sequences:", oeis.seq.len());

    let mut keys = oeis.seq.keys().cloned().collect::<Vec<_>>();
    keys.sort();

    keys = vec![961];

    for id in &keys {
        let nums = &oeis.seq[id].iter().map(|n| *n as u32).collect::<Vec<u32>>();

        if (1..nums.len()).any(|i| nums[i - 1] >= nums[i]) {
            continue;
        }

        println!("Searching A{:06}", id);

        let examples = (2u32..opts.max_val as u32).map(|n| (n, nums.contains(&n)));

        let output = simple_map(
            lang.clone(),
            examples,
            None,
            ty!(Var => Bool),
            SynthesisParameters {
                bias: SizeBias::DistAbs { mean: 23, c: 0.5 },
                iterations: 50000,
                ..Default::default()
            },
            Options {
                print_freq: Some(100),
                ..Default::default()
            },
        );

        if output.score.is_none() {
            use std::io::*;

            let term = output.term;
            let semantics = output.analysis.canon();

            let text = format!("Solution found for A{:06}: {} (≈ {})", id, term, semantics);

            println!("{}", text);
            writeln!(output_file, "{}", text)?;
            output_file.flush()?;
        }
    }

    Ok(())
}
