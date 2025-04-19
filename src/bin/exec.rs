use kolmogorov::*;

mod languages;
use languages::*;

fn main() {
    let lang = Polynomials;

    let term = term!(plus (plus one one));

    println!("out={:?}", term);
    let out = lang.context().evaluate(&term);
    println!("out={}", out);
}
