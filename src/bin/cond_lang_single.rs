mod languages;

use kolmogorov::*;
use languages::*;

fn main() {
    let lang = CondPolyLang;

    let nums = [1, 1, 3, 3, 5, 5, 7, 7, 9, 9];

    let examples = nums[1..].iter().cloned();

    let enumerated = (0i32..).zip(examples);

    let output = simple_map(
        lang,
        enumerated,
        Some(
            term!(n -> eval (case (div n (plus one one)) (plus one n) (orelse n)) ),
        ),
        ty!(Poly => N),
        SynthesisParameters {
            bias: SizeBias::DistAbs { mean: 30, c: 0.45 },
            iterations: 100_000,
            ..Default::default()
        },
        Options {
            print_freq: Some(100),
        },
    );

    output.display(lang);
}
