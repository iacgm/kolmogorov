use kolmogorov::*;

mod languages;
use languages::*;

fn main() {
    let lang = CondPolyLang;

    let term = term!(n -> eval (orelse n));

    let analysis = lang.analyze(&term);

    println!("Size: {}", term.size());
    println!("{}\n ≈ {}", term, analysis);
}
