use kolmogorov::*;

mod languages;
use languages::*;

fn main() {
    let lang = Polynomials;

    let num_examples = 10;

    let output = simple_map(
        lang,
        (0..num_examples).map(|n| (n, 4 * n * n * n + n * n)),
        None,
        ty!(N => N),
        SynthesisParameters {
            bias: SizeBias::DistAbs { mean: 20, c: 0.5 },
            ..Default::default()
        },
        Options::default(),
    );

    output.display(lang)
}
