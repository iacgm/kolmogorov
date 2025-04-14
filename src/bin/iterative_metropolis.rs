use kolmogorov::*;

mod languages;
mod utils;
use languages::*;

fn main() {
    let lang = Polynomials;
    let oeis = utils::oeis::load_oeis_def().unwrap();
    let nums = &oeis.seq[&142];

    let examples = nums[1..]
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, n)| ((i + 1) as i32, n));

    let output = iterative(
        lang,
        nums[0],
        examples,
        None,
        ty!(N => N => N),
        SynthesisParameters {
            bias: SizeBias::DistAbs { mean: 20, c: 0.5 },
            iterations: 75_000,
            ..Default::default()
        },
        Options { print_freq: None },
    );

    if output.score.is_none() {
        let term = output.term;
        let analysis = lang.analyze(&term);

        let text = format!("Solution found : {} (≈ {})", term, analysis);

        println!("{}", text);
    }
}
